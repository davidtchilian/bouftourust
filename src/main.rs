#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bouftourust::{
    assets::Manifest,
    data::GameData,
    profile::{Profile, SaveStore, SaveWorker},
    rendering::Textures,
    simulation::GamePlugin,
    ui::{UiState, interface},
};
use std::path::PathBuf;
fn assets_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("BOUFTOURUSH_ASSETS") {
        return p.into();
    }
    if let Ok(exe) = std::env::current_exe() {
        let p = exe.parent().unwrap().join("assets");
        if p.join("manifest.json").is_file() {
            return p;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}
fn main() {
    if let Err(e) = run() {
        eprintln!("BouftouRush : {e}");
        report_error(&e);
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let root = assets_dir();
    let data = GameData::load(&root)?;
    let manifest = Manifest::load(&root)?;
    manifest.validate(&data)?;
    let mut store = SaveStore::local();
    let smoke_mode = std::env::args().any(|a| a == "--smoke-frames");
    if smoke_mode {
        store.path =
            std::env::temp_dir().join(format!("bouftourush-smoke-{}.json", std::process::id()));
    }
    let args: Vec<_> = std::env::args().collect();
    if let Some(index) = args.iter().position(|a| a == "--import-profile") {
        let path = args
            .get(index + 1)
            .ok_or("Préciser le chemin du profil JSON exporté")?;
        let mut imported: Profile =
            serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        imported.serial = imported.serial.max(imported.inventory.len() as u64);
        imported.validate(&data)?;
        store.save(&imported)?;
        println!("Profil importé dans {}", store.path.display());
        return Ok(());
    }
    let mut profile = store.load(&data)?;
    let smoke_scene = args
        .iter()
        .position(|a| a == "--smoke-scene")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_default();
    if smoke_mode && smoke_scene == "inventory" {
        for item in data.items.values() {
            profile.give(&item.id);
            if item.set == "royalBouftou" {
                let uid = profile.inventory.last().unwrap().uid.clone();
                profile.equip(&uid, None, &data);
            }
        }
    }
    if args.iter().any(|a| a == "--validate") {
        profile.validate(&data)?;
        for a in manifest.assets.values() {
            for frame in &a.frames {
                if !root.join(&frame.path).is_file() {
                    return Err(format!("Asset manquant : {}", frame.path));
                }
            }
        }
        println!(
            "{} objets, {} monstres, {} donjons, {} animations. Sauvegarde : {}",
            data.items.len(),
            data.monsters.len(),
            data.dungeons.len(),
            manifest.assets.len(),
            store.path.display()
        );
        return Ok(());
    }
    let mut ui = UiState::default();
    ui.help = !profile.help_seen;
    ui.reduced_motion = profile.reduced_motion;
    if let Some(i) = args.iter().position(|a| a == "--smoke-scene") {
        if let Some(scene) = args.get(i + 1) {
            ui.help = false;
            ui.inventory = scene == "inventory";
            ui.dungeons = scene == "dungeons";
        }
    }
    let writer = SaveWorker::new(store.path.clone());
    let mut app = App::new();
    app.insert_resource(data)
        .insert_resource(manifest)
        .insert_resource(profile)
        .insert_resource(store)
        .insert_resource(writer)
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)))
        .insert_resource(ui)
        .init_resource::<Textures>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "BouftouRush · Local".into(),
                        resolution: (1360, 900).into(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(GamePlugin)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .add_systems(EguiPrimaryContextPass, interface);
    if smoke_mode && smoke_scene == "combat" {
        app.world_mut()
            .resource_mut::<bouftourust::simulation::Actions>()
            .0
            .push(bouftourust::simulation::Action::Enter(
                "bouftou".into(),
                true,
            ));
    }
    if let Some(index) = args.iter().position(|a| a == "--smoke-frames") {
        let limit = args
            .get(index + 1)
            .and_then(|a| a.parse::<u32>().ok())
            .unwrap_or(180);
        app.insert_resource(Smoke { frames: 0, limit })
            .add_systems(Update, smoke);
    }
    app.run();
    Ok(())
}
#[derive(Resource)]
struct Smoke {
    frames: u32,
    limit: u32,
}
fn smoke(
    mut smoke: ResMut<Smoke>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
    session: Res<bouftourust::simulation::Session>,
) {
    if session.loading {
        return;
    }
    smoke.frames += 1;
    if smoke.frames == smoke.limit {
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk("smoke.png"));
    }
    if smoke.frames > smoke.limit + 20 {
        exit.write(AppExit::Success);
    }
}

#[cfg(not(windows))]
fn report_error(_message: &str) {}
#[cfg(windows)]
fn report_error(message: &str) {
    use std::os::windows::ffi::OsStrExt;
    let text: Vec<u16> = std::ffi::OsStr::new(message)
        .encode_wide()
        .chain([0])
        .collect();
    let title: Vec<u16> = "BouftouRush — démarrage impossible"
        .encode_utf16()
        .chain([0])
        .collect();
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            window: *mut std::ffi::c_void,
            text: *const u16,
            title: *const u16,
            kind: u32,
        ) -> i32;
    }
    // SAFETY: chaînes UTF-16 terminées par NUL, conservées pendant l'appel ; aucune fenêtre parente.
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
    }
}
