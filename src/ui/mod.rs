pub use crate::ui_state::UiState;
use crate::{assets::*, data::*, profile::*, rendering::*, simulation::*};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
fn color() -> egui::Color32 {
    egui::Color32::from_rgb(184, 237, 191)
}
fn style(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = egui::Color32::from_rgb(14, 23, 25);
    v.window_fill = egui::Color32::from_rgb(23, 35, 38);
    v.extreme_bg_color = egui::Color32::from_rgb(14, 23, 25);
    v.selection.bg_fill = egui::Color32::from_rgb(42, 65, 53);
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 48, 52);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(43, 64, 66);
    v.widgets.active.bg_fill = egui::Color32::from_rgb(64, 91, 74);
    v.override_text_color = Some(egui::Color32::from_rgb(240, 244, 239));
    v.window_corner_radius = egui::CornerRadius::same(16);
    v.window_stroke = egui::Stroke::new(1_f32, egui::Color32::from_rgb(52, 72, 74));
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.button_padding = egui::vec2(12., 8.);
        s.spacing.item_spacing = egui::vec2(10., 8.);
    });
}
fn stat_value(value: f32) -> String {
    if value.fract().abs() < 0.001 {
        format!("{value:+.0}")
    } else {
        format!("{value:+.1}").replace('.', ",")
    }
}
fn stats_text(stats: &Stats) -> String {
    [
        (stats.hp, "Vie"),
        (stats.power, "Puissance"),
        (stats.movement_speed, "% vitesse"),
        (stats.cooldown_reduction, "% récupération"),
    ]
    .into_iter()
    .filter(|(value, _)| *value != 0.)
    .map(|(value, label)| format!("{} {label}", stat_value(value)))
    .collect::<Vec<_>>()
    .join("\n")
}
fn item_tooltip(ui: &mut egui::Ui, item: &Item, data: &GameData) {
    item_details(ui, &item.name, &item.stats, item, data);
}
fn instance_tooltip(ui: &mut egui::Ui, instance: &Instance, data: &GameData) {
    item_details(
        ui,
        &instance.name(data),
        &instance.stats(data),
        &data.items[&instance.item],
        data,
    );
}
fn item_details(ui: &mut egui::Ui, name: &str, stats: &Stats, item: &Item, data: &GameData) {
    ui.strong(name);
    ui.label(stats_text(stats));
    if let Some(set) = data.sets.get(&item.set) {
        ui.colored_label(color(), &set.name);
        for (n, stats) in &set.thresholds {
            ui.small(format!(
                "{n} pièces : {}",
                stats_text(stats).replace('\n', " · ")
            ));
        }
    } else {
        ui.small("Hors panoplie");
    }
}

mod inventory;
use inventory::inventory;
mod dungeons;
use dungeons::dungeons;
#[allow(clippy::too_many_arguments)]
pub fn interface(
    mut contexts: EguiContexts,
    mut textures: ResMut<Textures>,
    (server, images): (Res<AssetServer>, Res<Assets<Image>>),
    manifest: Res<Manifest>,
    data: Res<GameData>,
    profile: Res<Profile>,
    mut session: ResMut<Session>,
    mut state: ResMut<UiState>,
    mut actions: ResMut<Actions>,
    store: Res<SaveStore>,
    actors: Query<(Entity, &Actor, Option<&Enemy>)>,
    shots: Query<&Shot>,
    effects: Query<&Effect>,
    floating: Query<&Floating>,
    windows: Query<&Window>,
    time: Res<Time>,
) -> Result {
    let ctx = contexts.ctx_mut()?.clone();
    let previous_help = state.help;
    let previous_motion = state.reduced_motion;
    if !state.configured {
        style(&ctx);
        state.configured = true;
    }
    let (loaded, total, load_error) =
        textures.prepare(&session, &profile, &data, &manifest, &server, &mut contexts);
    session.loading = loaded < total || load_error.is_some();
    let mut art = Art {
        manifest: &manifest,
        textures: &mut textures,
        server: &server,
        contexts: &mut contexts,
    };
    let Some((_, player, _)) = actors.iter().find(|(e, _, _)| *e == session.player) else {
        return Ok(());
    };
    let stats = profile.stats(&data);
    if session.phase != Phase::Combat {
        state.manual_pause = false;
    }
    // Egui peut refaire la mise en page d'une même image : traiter les touches une fois.
    let frame = ctx.cumulative_frame_nr();
    let fresh_input = state.input_frame != Some(frame);
    state.input_frame = Some(frame);
    if fresh_input {
        ctx.input(|i| {
            let pressed=|key|i.events.iter().any(|event|matches!(event,egui::Event::Key{key:k,pressed:true,repeat:false,..} if *k==key));
            if pressed(egui::Key::I) {
                state.inventory = !state.inventory;
                state.dungeons = false;
                state.help = false;
            }
            if pressed(egui::Key::Escape) {
                state.escape(session.phase);
            }
            if pressed(egui::Key::F3) {
                state.fps = !state.fps;
            }
        });
    }
    egui::TopBottomPanel::bottom("hud")
        .exact_height(132.)
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(23, 35, 38))
                .inner_margin(14.),
        )
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(
                        color(),
                        format!(
                            "{} · NIVEAU {}",
                            data.classes[&profile.class_id].name, profile.level
                        ),
                    );
                    ui.strong(format!("{:.0} / {:.0} PV", player.hp, stats.hp));
                    ui.add(
                        egui::ProgressBar::new(player.hp / stats.hp)
                            .desired_width(175.)
                            .fill(egui::Color32::from_rgb(192, 71, 76)),
                    );
                    ui.add(
                        egui::ProgressBar::new(
                            profile.xp as f32 / xp_for_level(profile.level) as f32,
                        )
                        .desired_width(175.)
                        .text(format!(
                            "XP {} / {}",
                            profile.xp,
                            xp_for_level(profile.level)
                        ))
                        .fill(color()),
                    );
                });
                ui.separator();
                for (index, id) in data.classes[&profile.class_id].abilities.iter().enumerate() {
                    let spell = &data.spells[id];
                    let remaining = (session.cooldowns[index] - session.time).max(0.);
                    ui.vertical(|ui| {
                        let response = art
                            .spell_icon(
                                ui,
                                &format!("ui.spell.{id}"),
                                egui::vec2(38., 38.),
                                remaining > 0.,
                                &images,
                            )
                            .on_hover_text(
                                if ["sournoiserie", "mass_trap", "invisibility", "mortal_attack"]
                                    .contains(&id.as_str())
                                {
                                    format!(
                                        "{}\n{}\nRecharge : {:.2} s",
                                        spell.name,
                                        data.spell_descriptions[id],
                                        spell.cooldown * (1. - stats.cooldown_reduction / 100.)
                                    )
                                } else if id == "iop_power" {
                                    format!(
                                        "Puissance\nDégâts ×2 pendant 5 s\nRecharge : {:.2} s",
                                        spell.cooldown * (1. - stats.cooldown_reduction / 100.)
                                    )
                                } else {
                                    format!(
                                        "{}\nRecharge : {:.2} s\nDégâts ×{:.1} · Portée {:.0}",
                                        spell.name,
                                        spell.cooldown * (1. - stats.cooldown_reduction / 100.),
                                        spell.coefficient,
                                        spell.range.unwrap_or(f32::INFINITY)
                                    ) + if id == "wrath" {
                                        "\nZone autour du curseur · rayon 60"
                                    } else {
                                        ""
                                    }
                                },
                            );
                        if response.clicked() {
                            actions.0.push(Action::Cast(index, state.aim));
                        }
                        ui.label(
                            egui::RichText::new(["A", "Z", "E", "R"][index])
                                .strong()
                                .color(if remaining > 0. {
                                    egui::Color32::GRAY
                                } else {
                                    color()
                                }),
                        );
                        ui.small(match id.as_str() {
                            "explosive" => "Explosive",
                            "flaming" => "Enflammée",
                            "magic" => "Magique",
                            "power" => "Puissant",
                            _ => spell.name.as_str(),
                        });
                        ui.small(if remaining > 0. {
                            format!("{remaining:.1} s")
                        } else {
                            "Prêt".into()
                        });
                    });
                }
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_max_width(300.);
                    ui.horizontal_wrapped(|ui| {
                        ui.strong(
                            session
                                .dungeon
                                .as_ref()
                                .map(|id| data.dungeons[id].name.as_str())
                                .unwrap_or("Astrub · Zaap"),
                        );
                        ui.colored_label(color(), format!("{} kamas", profile.kamas));
                        if session.phase == Phase::Hub {
                            egui::ComboBox::from_id_salt("class")
                                .selected_text(&data.classes[&profile.class_id].name)
                                .show_ui(ui, |ui| {
                                    for c in data.classes.values() {
                                        if ui
                                            .selectable_label(c.id == profile.class_id, &c.name)
                                            .clicked()
                                        {
                                            actions.0.push(Action::Class(c.id.clone()));
                                        }
                                    }
                                });
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if session.phase == Phase::Hub && ui.button("Donjons").clicked() {
                            state.dungeons = true;
                            state.inventory = false;
                            state.help = false;
                        }
                        if ui.button("Inventaire (I)").clicked() {
                            state.inventory = !state.inventory;
                            state.dungeons = false;
                            state.help = false;
                        }
                        if session.phase == Phase::Combat && ui.button("Abandonner").clicked() {
                            actions.0.push(Action::Hub);
                        }
                        if ui.button("Aide").clicked() {
                            state.help = !state.help;
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if session.phase == Phase::Combat {
                            let living = actors
                                .iter()
                                .filter(|(_, a, e)| e.is_some() && a.hp > 0.)
                                .count();
                            ui.small(format!(
                                "Vague {}/4 · {living} ennemis{}",
                                (session.wave + 1).max(1),
                                if session.extreme { " · Extrême" } else { "" }
                            ));
                            if living == 0 {
                                ui.small(format!(
                                    "Prochaine vague : {:.1} s",
                                    session.delay.max(0.)
                                ));
                            }
                        }
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_min_width(220.);
                    ui.label(egui::RichText::new("Effets actifs").size(18.).strong());
                    let mut statuses = Vec::new();
                    if session.phase == Phase::Combat {
                        if session.buff_until > session.time {
                            let (name, multiplier) = if profile.class_id == "iop" {
                                (
                                    "Puissance",
                                    data.spells["iop_power"].buff_multiplier.unwrap_or(2.),
                                )
                            } else {
                                ("Tir puissant", data.balance.buff_multiplier)
                            };
                            statuses.push(format!(
                                "{name} ×{multiplier:.1} · {:.1} s",
                                session.buff_until - session.time
                            ));
                        }
                        if session.truce_until > session.time {
                            statuses.push(format!(
                                "Trêve · {:.1} s",
                                session.truce_until - session.time
                            ));
                        }
                        if profile.class_id == "sram" {
                            if session.invisible_until > session.time {
                                statuses.push(format!(
                                    "Invisible · {:.1} s",
                                    session.invisible_until - session.time
                                ));
                            }
                            if !session.traps.is_empty() {
                                statuses.push(format!("{} pièges actifs", session.traps.len()));
                            }
                        }
                        if profile.class_id == "sacrier" {
                            if session.chastisement_until > session.time {
                                statuses.push(format!(
                                    "Châtiment · {:.1} s",
                                    session.chastisement_until - session.time
                                ));
                            }
                            if session.combat_power > 0. {
                                statuses.push(format!(
                                    "+{:.0} Puissance acquise",
                                    session.combat_power
                                ));
                            }
                        }
                        if profile.class_id == "iop" {
                            statuses.push(format!("Prochaine Colère ×{:.2}", session.wrath));
                        }
                        if profile.class_id == "feca" && !session.glyphs.is_empty() {
                            statuses.push(format!("{} glyphes actifs", session.glyphs.len()));
                        }
                    }
                    if statuses.is_empty() {
                        ui.label(egui::RichText::new("Aucun effet actif").size(16.).weak());
                    } else {
                        for status in statuses {
                            ui.label(
                                egui::RichText::new(status)
                                    .size(16.)
                                    .strong()
                                    .color(color()),
                            );
                        }
                    }
                });
            });
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(11, 19, 21)))
        .show(&ctx, |ui| {
            let rect = ui.max_rect();
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            let view = View::new(rect, session.map(&manifest));
            let actor_list: Vec<_> = actors
                .iter()
                .map(|(e, a, m)| (e, a.clone(), m.cloned()))
                .collect();
            world(
                &mut art,
                ui.painter(),
                view,
                &session,
                &profile,
                &data,
                &actor_list,
                &shots.iter().cloned().collect::<Vec<_>>(),
                &effects.iter().cloned().collect::<Vec<_>>(),
                &floating.iter().cloned().collect::<Vec<_>>(),
            );
            if let Some(pos) = response.hover_pos() {
                state.aim = view.ground(pos); // Viser une silhouette vise ses pieds logiques.
                let hovered = actor_list
                    .iter()
                    .filter(|(_, a, m)| m.is_some() && a.hp > 0.)
                    .filter(|(_, a, enemy)| {
                        let enemy = enemy.as_ref().unwrap();
                        let def = &data.monsters[&enemy.kind];
                        let (side, flip) = facing(a.direction);
                        let id = format!("mob.{}.{}.{side}", def.sprite, a.action);
                        let Some(asset) = manifest.assets.get(&id) else {
                            return false;
                        };
                        let feet = view.project(a.position + Vec2::from(asset.offset));
                        let left = if flip {
                            -(1. - asset.pivot[0]) * asset.size[0]
                        } else {
                            -asset.pivot[0] * asset.size[0]
                        };
                        let bounds = egui::Rect::from_min_size(
                            feet + egui::vec2(left, -asset.pivot[1] * asset.size[1]) * view.scale,
                            egui::vec2(asset.size[0], asset.size[1]) * view.scale,
                        );
                        bounds.contains(pos)
                    })
                    .max_by(|(_, a, _), (_, b, _)| {
                        s_depth(&session, a).total_cmp(&s_depth(&session, b))
                    });
                if let Some((_, a, _)) = hovered {
                    state.aim = a.position;
                }
                let focused = windows.iter().any(|w| w.focused);
                if !state.inventory
                    && !state.dungeons
                    && focused
                    && ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary))
                {
                    actions.0.push(Action::Move(view.ground(pos)));
                }
            }
            if state.fps {
                ui.painter().text(
                    rect.left_top() + egui::vec2(12., 12.),
                    egui::Align2::LEFT_TOP,
                    format!(
                        "{:.0} FPS · {} acteurs · {} textures · {} kills",
                        1. / time.delta_secs().max(0.001),
                        actor_list.len(),
                        art.textures.images.len(),
                        session.kills
                    ),
                    egui::FontId::monospace(12.),
                    egui::Color32::WHITE,
                );
            }
            let mut y = rect.bottom() - 12.;
            for line in session.log.iter().take(2) {
                ui.painter().text(
                    egui::pos2(rect.left() + 16., y),
                    egui::Align2::LEFT_BOTTOM,
                    line,
                    egui::FontId::proportional(13.),
                    egui::Color32::from_white_alpha(210),
                );
                y -= 19.;
            }
        });
    if fresh_input && !state.pauses_game() && !ctx.wants_keyboard_input() {
        for (index, key) in [egui::Key::A, egui::Key::Z, egui::Key::E, egui::Key::R]
            .iter()
            .enumerate()
        {
            if ctx.input(|i| i.events.iter().any(|event|matches!(event,egui::Event::Key{key:k,pressed:true,repeat:false,..} if k==key))) {
                actions.0.push(Action::Cast(index, state.aim));
            }
        }
    }
    if state.pauses_game() {
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("veil"),
        ));
        painter.rect_filled(ctx.content_rect(), 0., egui::Color32::from_black_alpha(145));
    }
    if state.inventory {
        inventory(
            &ctx,
            &mut art,
            &mut state,
            &profile,
            &data,
            &session,
            &mut actions,
        );
    }
    if state.dungeons {
        dungeons(&ctx, &mut art, &mut state, &profile, &data, &mut actions);
    }
    if state.help {
        let mut open = true;
        egui::Window::new("À vous de jouer").open(&mut open).collapsible(false).default_width(390.).anchor(egui::Align2::RIGHT_TOP,[-20.,20.]).show(&ctx,|ui|{ui.colored_label(color(),"BOUFTOURUSH · ÉDITION LOCALE");ui.label("Clic droit : déplacement. Maintenez pour suivre le curseur. Relâcher conserve la destination.");ui.label("A Z E R : lancez vos sorts vers le curseur.");ui.label("I : inventaire, équipements et panoplies. Échap : fermer les fenêtres.");ui.label("Choisissez un donjon. Trois vagues, puis le boss. Les gains restent acquis après une défaite.");ui.checkbox(&mut state.reduced_motion,"Réduire les animations des aperçus");if ui.button("Compris").clicked(){state.help=false;}});
        state.help &= open;
    }
    if matches!(session.phase, Phase::Victory | Phase::Death) && !state.inventory {
        egui::Window::new(if session.phase == Phase::Victory {
            "Donjon terminé !"
        } else {
            "Vous êtes tombé"
        })
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., -40.])
        .show(&ctx, |ui| {
            ui.label("Vos XP, kamas et objets sont conservés.");
            ui.label(format!(
                "Niveau {} · {} kamas · {} objets",
                profile.level,
                profile.kamas,
                profile.inventory.len()
            ));
            if ui.button("Retour à Astrub").clicked() {
                actions.0.push(Action::Hub);
            }
            if let Some(id) = &session.dungeon {
                if ui.button("Recommencer").clicked() {
                    actions.0.push(Action::Enter(id.clone(), session.extreme));
                }
            }
            if ui.button("Voir l’inventaire").clicked() {
                state.inventory = true;
            }
        });
    }
    if state.manual_pause
        && session.phase == Phase::Combat
        && !state.inventory
        && !state.dungeons
        && !state.help
    {
        egui::Window::new("Pause")
            .id(egui::Id::new("combat-pause"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0., -40.])
            .show(&ctx, |ui| {
                ui.label("Le combat est en pause.");
                if ui.button("Reprendre (Échap)").clicked() {
                    state.manual_pause = false;
                }
            });
    }
    if let Some(error) = &store.error {
        egui::Window::new("Enregistrement impossible").show(&ctx, |ui| {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
            ui.label("Le fichier existant est conservé.");
        });
    }
    if session.loading {
        egui::Window::new("Chargement local")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0., -50.])
            .show(&ctx, |ui| {
                ui.label(format!("Textures depuis le disque : {loaded} / {total}"));
                ui.add(egui::ProgressBar::new(loaded as f32 / total.max(1) as f32));
                if let Some(error) = &load_error {
                    ui.colored_label(egui::Color32::LIGHT_RED, error);
                    ui.label("Voir docs/ASSETS.md pour restaurer les fichiers.");
                }
            });
    }
    if (previous_help && !state.help) || previous_motion != state.reduced_motion {
        actions.0.push(Action::Preferences {
            help_seen: profile.help_seen || (previous_help && !state.help),
            reduced_motion: state.reduced_motion,
        });
    }
    session.paused = state.pauses_game();
    Ok(())
}
fn s_depth(s: &Session, a: &Actor) -> f32 {
    s.navigation.depth(a.position)
}
