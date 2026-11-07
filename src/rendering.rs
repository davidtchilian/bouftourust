//! Quads 2D texturés sur le renderer wgpu de Bevy via egui.
//! Les transformations affines Flash sont conservées, y compris cisaillement et miroirs.
use crate::{assets::*, data::*, profile::*, simulation::*};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};
use std::collections::{HashMap, HashSet};
#[derive(Resource, Default)]
pub struct Textures {
    pub images: HashMap<String, (Handle<Image>, egui::TextureId)>,
    last_used: HashMap<String, u64>,
    frame: u64,
    signature: String,
    pinned: HashSet<String>,
    grayscale_icons: HashMap<String, egui::TextureHandle>,
}
impl Textures {
    pub fn get(
        &mut self,
        path: &str,
        server: &AssetServer,
        contexts: &mut EguiContexts,
    ) -> egui::TextureId {
        self.last_used.insert(path.into(), self.frame);
        self.images
            .entry(path.into())
            .or_insert_with(|| {
                let h: Handle<Image> = server.load(path.to_owned());
                let id = contexts.add_image(EguiTextureHandle::Strong(h.clone()));
                (h, id)
            })
            .1
    }
    pub fn prepare(
        &mut self,
        s: &Session,
        p: &Profile,
        d: &GameData,
        m: &Manifest,
        server: &AssetServer,
        contexts: &mut EguiContexts,
    ) -> (usize, usize, Option<String>) {
        self.frame += 1;
        let signature = format!("{}:{}:{:?}", s.revision, p.class_id, p.equipment);
        if signature != self.signature {
            self.signature = signature;
            self.pinned.clear();
            let map = s.map(m);
            let mut ids: HashSet<String> = map
                .layers
                .iter()
                .filter(|l| map.background_asset_id.is_none() || l.order >= 2000.)
                .map(|l| l.asset_id.clone())
                .collect();
            if let Some(id) = &map.background_asset_id {
                ids.insert(id.clone());
            }
            for (id, pose) in &m.composition.actors {
                if !id.starts_with(&format!("{}.", p.class_id)) {
                    continue;
                }
                for frame in &pose.frames {
                    for part in &frame.layers {
                        let base = part.asset_id.as_deref().or_else(|| {
                            part.slot
                                .as_ref()
                                .and_then(|slot| {
                                    if slot == "bow" {
                                        Some("bow")
                                    } else {
                                        p.worn(slot)
                                    }
                                })
                                .and_then(|item| m.composition.accessories.get(item))
                                .and_then(|a| a.views.get(part.label.as_deref().unwrap_or("")))
                                .map(String::as_str)
                        });
                        if let Some(base) = base {
                            ids.insert(
                                part.masked_assets
                                    .get(base)
                                    .map(String::as_str)
                                    .unwrap_or(base)
                                    .into(),
                            );
                        }
                    }
                }
            }
            let class = &d.classes[&p.class_id];
            if p.class_id == "sram" {
                ids.insert("spell.glyph.zone".into());
            }
            for id in &class.abilities {
                let spell = &d.spells[id];
                ids.insert(format!("ui.spell.{id}"));
                if !spell.effect.is_empty() {
                    ids.insert(spell.effect.clone());
                }
                let shot = format!("spell.{id}.projectile");
                if m.assets.contains_key(&shot) {
                    ids.insert(shot);
                }
            }
            if let Some(id) = &s.dungeon {
                let dungeon = &d.dungeons[id];
                let kinds: HashSet<_> = dungeon
                    .normal_waves
                    .iter()
                    .flatten()
                    .chain(&dungeon.boss_wave)
                    .collect();
                for kind in kinds {
                    let def = &d.monsters[kind];
                    if def.invulnerability.is_some() {
                        ids.insert("spell.truce.cast".into());
                    }
                    let prefix = format!("mob.{}.", def.sprite);
                    for id in m.assets.keys().filter(|id| id.starts_with(&prefix)) {
                        ids.insert(id.clone());
                    }
                    if let Some(r) = &def.ranged_attack {
                        ids.insert(r.asset.clone());
                        for side in ["L", "R"] {
                            ids.insert(format!("{}.{side}", r.animation));
                        }
                    }
                }
            }
            for id in ids {
                if let Some(a) = m.assets.get(&id) {
                    for f in &a.frames {
                        self.pinned.insert(f.path.clone());
                    }
                }
            }
        }
        for path in self.pinned.clone() {
            self.get(&path, server, contexts);
        }
        let expired: Vec<_> = self
            .images
            .keys()
            .filter(|path| {
                !self.pinned.contains(*path)
                    && self
                        .frame
                        .saturating_sub(*self.last_used.get(*path).unwrap_or(&0))
                        > 300
            })
            .cloned()
            .collect();
        for path in expired {
            if let Some((h, _)) = self.images.remove(&path) {
                contexts.remove_image(h.id());
            }
            self.last_used.remove(&path);
        }
        let mut loaded = 0;
        let mut failure = None;
        for path in &self.pinned {
            let h = &self.images[path].0;
            match server.get_load_state(h.id()) {
                Some(bevy::asset::LoadState::Loaded) => loaded += 1,
                Some(bevy::asset::LoadState::Failed(e)) => {
                    failure = Some(format!("Asset illisible : {path} : {e}"))
                }
                _ => {}
            }
        }
        (loaded, self.pinned.len(), failure)
    }
}
#[derive(Clone, Copy)]
pub struct View {
    pub rect: egui::Rect,
    pub center: Vec2,
    pub scale: f32,
}
impl View {
    pub fn new(rect: egui::Rect, map: &Map) -> Self {
        let [l, b, r, t] = map.bounds;
        let height = map
            .camera_height
            .max(t - b)
            .max((r - l) * rect.height() / rect.width().max(1.));
        Self {
            rect,
            center: Vec2::new((l + r) / 2., (b + t) / 2.),
            scale: rect.height() / height,
        }
    }
    pub fn project(&self, p: Vec2) -> egui::Pos2 {
        egui::pos2(
            self.rect.center().x + (p.x - self.center.x) * self.scale,
            self.rect.center().y - (p.y - self.center.y) * self.scale,
        )
    }
    pub fn ground(&self, p: egui::Pos2) -> Vec2 {
        self.center
            + Vec2::new(
                (p.x - self.rect.center().x) / self.scale,
                -(p.y - self.rect.center().y) / self.scale,
            )
    }
}
pub struct Art<'a, 'w, 's> {
    pub manifest: &'a Manifest,
    pub textures: &'a mut Textures,
    pub server: &'a AssetServer,
    pub contexts: &'a mut EguiContexts<'w, 's>,
}
impl Art<'_, '_, '_> {
    pub fn spell_icon(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        size: egui::Vec2,
        cooldown: bool,
        images: &Assets<Image>,
    ) -> egui::Response {
        if !cooldown {
            return self.icon(ui, id, size);
        }
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let Some(asset) = self.manifest.assets.get(id) else {
            return response;
        };
        let frame = &asset.frames[0];
        self.textures.get(&frame.path, self.server, self.contexts);
        if !self.textures.grayscale_icons.contains_key(id) {
            let handle = &self.textures.images[&frame.path].0;
            if let Some(image) = images.get(handle) {
                if let Some(bytes) = &image.data {
                    let width = image.width() as usize;
                    let height = image.height() as usize;
                    if bytes.len() == width * height * 4 {
                        let [x, y, w, h] = frame
                            .region
                            .map(|r| [r[0], r[1], r[2], r[3]])
                            .unwrap_or([0, 0, width as u32, height as u32]);
                        let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
                        for row in y..y + h {
                            for col in x..x + w {
                                let i = (row as usize * width + col as usize) * 4;
                                let gray = ((u32::from(bytes[i]) * 54
                                    + u32::from(bytes[i + 1]) * 183
                                    + u32::from(bytes[i + 2]) * 19)
                                    / 256) as u8;
                                rgba.extend_from_slice(&[gray, gray, gray, bytes[i + 3]]);
                            }
                        }
                        let texture = ui.ctx().load_texture(
                            format!("cooldown:{id}"),
                            egui::ColorImage::from_rgba_unmultiplied(
                                [w as usize, h as usize],
                                &rgba,
                            ),
                            egui::TextureOptions::LINEAR,
                        );
                        self.textures.grayscale_icons.insert(id.into(), texture);
                    }
                }
            }
        }
        if let Some(texture) = self.textures.grayscale_icons.get(id) {
            let scale = (rect.width() / asset.size[0]).min(rect.height() / asset.size[1]);
            let fit = egui::Rect::from_center_size(
                rect.center(),
                egui::vec2(asset.size[0], asset.size[1]) * scale,
            );
            ui.painter().image(
                texture.id(),
                fit,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                egui::Color32::WHITE,
            );
        }
        response
    }
    pub fn quad(
        &mut self,
        painter: &egui::Painter,
        id: &str,
        frame: usize,
        origin: egui::Pos2,
        scale: f32,
        m: [f32; 6],
        mirror: bool,
        opacity: f32,
    ) {
        let Some(a) = self.manifest.assets.get(id) else {
            return;
        };
        let f = &a.frames[frame % a.frames.len()];
        let texture = self.textures.get(&f.path, self.server, self.contexts);
        let uv = if let Some([x, y, w, h, pw, ph]) = f.region {
            [
                x as f32 / pw as f32,
                y as f32 / ph as f32,
                (x + w) as f32 / pw as f32,
                (y + h) as f32 / ph as f32,
            ]
        } else {
            [0., 0., 1., 1.]
        };
        let mut mesh = egui::Mesh::with_texture(texture);
        let px = -a.pivot[0] * a.size[0];
        let py = -a.pivot[1] * a.size[1];
        let corners = [
            [px, py],
            [px + a.size[0], py],
            [px + a.size[0], py + a.size[1]],
            [px, py + a.size[1]],
        ];
        let uvs = [
            [uv[0], uv[1]],
            [uv[2], uv[1]],
            [uv[2], uv[3]],
            [uv[0], uv[3]],
        ];
        let flip = if mirror { -1. } else { 1. };
        for (i, [x, y]) in corners.into_iter().enumerate() {
            let dx = flip * (m[0] * x + m[2] * y + m[4]);
            let dy = m[1] * x + m[3] * y + m[5];
            mesh.vertices.push(egui::epaint::Vertex {
                pos: origin + egui::vec2(dx * scale, dy * scale),
                uv: egui::pos2(uvs[i][0], uvs[i][1]),
                color: egui::Color32::from_white_alpha((opacity * 255.) as u8),
            });
        }
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        painter.add(egui::Shape::mesh(mesh));
    }
    pub fn sprite(
        &mut self,
        painter: &egui::Painter,
        id: &str,
        age: f32,
        origin: egui::Pos2,
        scale: f32,
        mirror: Option<bool>,
        angle: f32,
        window: Option<f32>,
    ) {
        if let Some(a) = self.manifest.assets.get(id) {
            let index = a.index(age, window);
            let (c, s) = (angle.cos(), angle.sin());
            self.quad(
                painter,
                id,
                index,
                origin + egui::vec2(a.offset[0] * scale, -a.offset[1] * scale),
                scale,
                [c, -s, s, c, 0., 0.],
                mirror.unwrap_or(a.flip_x),
                1.,
            );
        }
    }
    pub fn icon(&mut self, ui: &mut egui::Ui, id: &str, size: egui::Vec2) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        self.in_rect(ui.painter(), id, 0., rect, false);
        response
    }
    pub fn in_rect(
        &mut self,
        painter: &egui::Painter,
        id: &str,
        age: f32,
        rect: egui::Rect,
        animated: bool,
    ) {
        let Some(a) = self.manifest.assets.get(id) else {
            return;
        };
        let scale = (rect.width() / a.size[0]).min(rect.height() / a.size[1]);
        let origin = rect.center()
            + egui::vec2(
                (a.pivot[0] - 0.5) * a.size[0] * scale,
                (a.pivot[1] - 0.5) * a.size[1] * scale,
            );
        self.sprite(
            painter,
            id,
            if animated {
                age % (a.frames.iter().map(|f| f.duration_ms).sum::<f32>() / 1000. + 1.6)
            } else {
                0.
            },
            origin,
            scale,
            None,
            0.,
            None,
        );
    }
    pub fn actor(
        &mut self,
        painter: &egui::Painter,
        id: &str,
        p: &Profile,
        age: f32,
        origin: egui::Pos2,
        scale: f32,
        mirror: Option<bool>,
        window: Option<f32>,
        hd: bool,
        opacity: f32,
    ) {
        let Some(pose) = self.manifest.composition.actors.get(id) else {
            return;
        };
        let a = &self.manifest.assets[id];
        let i = a.index(age, window);
        let flip = mirror.unwrap_or(pose.flip_x);
        let hat = p.worn("hat").unwrap_or("none");
        for part in &pose.frames[i % pose.frames.len()].layers {
            if (part.hide_with_hat && hat != "none") || (part.show_with_hat && hat == "none") {
                continue;
            }
            let mut asset = part.asset_id.as_deref();
            let mut index = 0;
            if let Some(slot) = &part.slot {
                let item = if slot == "bow" {
                    "bow"
                } else {
                    p.worn(slot).unwrap_or("none")
                };
                if item == "none" {
                    continue;
                }
                asset = self
                    .manifest
                    .composition
                    .accessories
                    .get(item)
                    .and_then(|accessory| accessory.views.get(part.label.as_deref().unwrap_or("")))
                    .map(String::as_str);
                index = i;
            }
            if let Some(id) = asset {
                let masked = part.masked_assets.get(id).map(String::as_str);
                let id = masked.unwrap_or(id);
                let preview = format!("preview.{id}");
                let id = if hd && self.manifest.assets.contains_key(&preview) {
                    preview.as_str()
                } else {
                    id
                };
                self.quad(
                    painter,
                    id,
                    if masked.is_some() { 0 } else { index },
                    origin,
                    scale,
                    part.matrix,
                    flip,
                    part.opacity * opacity,
                );
            }
        }
    }
}
pub fn player_clip(a: &Actor, p: &Profile) -> (String, Option<bool>) {
    if a.action == "idle" || a.action == "run" {
        (
            format!("{}.{}.{}", p.class_id, a.action, DIRECTIONS[a.direction]),
            None,
        )
    } else {
        let (side, flip) = facing(a.direction);
        (format!("{}.{}.{side}", p.class_id, a.action), Some(flip))
    }
}
pub fn world(
    art: &mut Art,
    painter: &egui::Painter,
    view: View,
    s: &Session,
    p: &Profile,
    d: &GameData,
    actors: &[(Entity, Actor, Option<Enemy>)],
    shots: &[Shot],
    effects: &[Effect],
    floating: &[Floating],
) {
    let map = s.map(art.manifest);
    let [l, b, r, t] = map.bounds;
    let map_rect =
        egui::Rect::from_two_pos(view.project(Vec2::new(l, t)), view.project(Vec2::new(r, b)));
    let painter = painter.with_clip_rect(view.rect.intersect(map_rect));
    let active_buff = if s.phase != Phase::Combat {
        None
    } else if p.class_id == "sacrier" && s.time < s.chastisement_until {
        Some((
            "chastisement",
            s.chastisement_until,
            d.class_balance.chastisement_duration,
            0.4,
            0.45,
        ))
    } else if p.class_id == "iop" && s.time < s.buff_until {
        Some((
            "iop_power",
            s.buff_until,
            d.spells["iop_power"].buff_duration.unwrap_or(5.),
            1.,
            0.8,
        ))
    } else if p.class_id == "sram" && s.time < s.invisible_until {
        Some((
            "invisibility",
            s.invisible_until,
            d.spells["invisibility"].buff_duration.unwrap_or(4.),
            0.,
            0.,
        ))
    } else {
        None
    };
    struct Draw {
        z: f32,
        id: String,
        point: Vec2,
        age: f32,
        flip: Option<bool>,
        angle: f32,
        scale: Vec2,
        window: Option<f32>,
        player: bool,
    }
    let mut draws =
        Vec::with_capacity(map.layers.len() + actors.len() + shots.len() + effects.len());
    if let Some(id) = &map.background_asset_id {
        draws.push(Draw {
            z: 1999.,
            id: id.clone(),
            point: Vec2::new(l, t),
            age: 0.,
            flip: None,
            angle: 0.,
            scale: Vec2::ONE,
            window: None,
            player: false,
        });
    }
    for layer in &map.layers {
        if map.background_asset_id.is_some() && layer.order < 2000. {
            continue;
        }
        draws.push(Draw {
            z: layer.order,
            id: layer.asset_id.clone(),
            point: layer.position.into(),
            age: 0.,
            flip: Some(layer.flip_x),
            angle: layer.rotation.to_radians(),
            scale: layer.scale.into(),
            window: None,
            player: false,
        });
    }
    for (_, a, e) in actors {
        if a.hp > 0.
            && s.phase == Phase::Combat
            && (e.is_none() && s.time < s.truce_until
                || e.as_ref().is_some_and(|enemy| {
                    d.monsters[&enemy.kind]
                        .invulnerability
                        .as_ref()
                        .is_some_and(|shield| shield.active(s.time - enemy.spawned))
                }))
        {
            draws.push(Draw {
                z: 2000. + s.navigation.depth(a.position) + 0.4,
                id: "status.invulnerable".into(),
                point: a.position + Vec2::Y * 20.,
                age: s.time,
                flip: None,
                angle: 0.,
                scale: Vec2::ONE,
                window: None,
                player: false,
            });
        }
        let (id, flip, window) = if let Some(enemy) = e {
            let def = &d.monsters[&enemy.kind];
            let (side, flip) = facing(a.direction);
            let ranged = def
                .ranged_attack
                .as_ref()
                .filter(|_| a.action == "attack" && s.time < enemy.ranged_until);
            let id = if let Some(r) = ranged {
                format!("{}.{side}", r.animation)
            } else {
                format!("mob.{}.{}.{side}", def.sprite, a.action)
            };
            let window = if let Some(r) = ranged {
                Some(r.animation_duration)
            } else {
                match a.action {
                    "attack" => Some(d.balance.enemy_attack_animation),
                    "hit" => Some(d.balance.enemy_hit_animation),
                    "death" => Some(d.balance.enemy_death_animation),
                    _ => None,
                }
            };
            (id, Some(flip), window)
        } else {
            let (id, flip) = player_clip(a, p);
            (
                id,
                flip,
                if a.action == "cast" || a.action == "buff" {
                    Some(d.balance.cast_animation)
                } else {
                    None
                },
            )
        };
        draws.push(Draw {
            z: 2000. + s.navigation.depth(a.position) + if e.is_some() { 0.3 } else { 0.5 },
            id,
            point: a.position,
            age: a.animation,
            flip,
            angle: 0.,
            scale: Vec2::ONE,
            window,
            player: e.is_none(),
        });
    }
    if !s.glyphs.is_empty() {
        for cell in &s.navigation.cells {
            let point = Vec2::from(cell.position);
            if s.glyphs
                .iter()
                .any(|glyph| point.distance(glyph.position) <= d.spells["glyph"].radius)
            {
                draws.push(Draw {
                    // Above the baked ground (1999), below scenery and actors (2000+).
                    z: 1999.5,
                    id: "spell.glyph.zone".into(),
                    point,
                    age: s.time,
                    flip: None,
                    angle: 0.,
                    scale: Vec2::ONE,
                    window: None,
                    player: false,
                });
            }
        }
    }
    let mut trap_cells = HashMap::new();
    for trap in &s.traps {
        let key = (trap.position.x.to_bits(), trap.position.y.to_bits());
        *trap_cells.entry(key).or_insert(0_usize) += 1;
    }
    for (&(x, y), _) in &trap_cells {
        draws.push(Draw {
            z: 1999.6,
            id: "status.trap".into(),
            point: Vec2::new(f32::from_bits(x), f32::from_bits(y)),
            age: 0.,
            flip: None,
            angle: 0.,
            scale: Vec2::ONE,
            window: None,
            player: false,
        });
    }
    for shot in shots {
        draws.push(Draw {
            z: 5000.,
            id: shot.asset.clone(),
            point: if shot.spell == "flaming" {
                shot.start
            } else {
                shot.position
            },
            age: shot.age,
            flip: Some(false),
            angle: (shot.target - shot.start).to_angle(),
            scale: Vec2::ONE,
            window: None,
            player: false,
        });
    }
    for e in effects {
        draws.push(Draw {
            z: 5100.,
            id: e.asset.clone(),
            point: e.position,
            age: e.age,
            flip: Some(false),
            angle: if e.asset.starts_with("spell.flaming") {
                e.angle
            } else {
                0.
            },
            scale: if e.asset == "spell.sram.invisibility" {
                Vec2::splat(0.15)
            } else {
                Vec2::ONE
            },
            window: None,
            player: false,
        });
    }
    draws.sort_by(|a, b| a.z.total_cmp(&b.z));
    for draw in draws {
        let origin = view.project(draw.point);
        if draw.player {
            art.actor(
                &painter,
                &draw.id,
                p,
                draw.age,
                origin,
                view.scale,
                draw.flip,
                draw.window,
                false,
                if s.time < s.invisible_until { 0.3 } else { 1. },
            );
            if let Some((spell, until, duration, scale, opacity)) =
                active_buff.filter(|(_, _, _, scale, _)| *scale > 0.)
            {
                let id = &d.spells[spell].effect;
                if let Some(animation) = art.manifest.assets.get(id) {
                    let elapsed = s.time - (until - duration);
                    let frame = frame_index(&animation.frames, elapsed, true, None);
                    art.quad(
                        &painter,
                        id,
                        frame,
                        view.project(draw.point + Vec2::Y * 20.),
                        view.scale * scale,
                        [1., 0., 0., 1., 0., 0.],
                        false,
                        opacity,
                    );
                }
            }
        } else if draw.id == "spell.sram.trap" {
            let animation = &art.manifest.assets[&draw.id];
            let frame = animation.index(draw.age, None);
            let scale = d.spells["mass_trap"].radius / animation.size[0];
            for ray in 0..6 {
                let angle = std::f32::consts::TAU * ray as f32 / 6.;
                let (c, s) = (angle.cos() * scale, angle.sin() * scale);
                art.quad(
                    &painter,
                    &draw.id,
                    frame,
                    origin,
                    view.scale,
                    [c, -s, s, c, 0., 0.],
                    false,
                    1.,
                );
            }
        } else if draw.id == "status.trap" {
            art.quad(
                &painter,
                "spell.glyph.zone",
                0,
                origin,
                view.scale,
                [1., 0., 0., 1., 0., 0.],
                false,
                0.65,
            );
            let count = trap_cells[&(draw.point.x.to_bits(), draw.point.y.to_bits())];
            if count > 1 {
                painter.text(
                    origin,
                    egui::Align2::CENTER_CENTER,
                    format!("×{count}"),
                    egui::FontId::proportional(13.),
                    egui::Color32::WHITE,
                );
            }
        } else if draw.id == "status.invulnerable" {
            let id = "spell.truce.cast";
            if let Some(animation) = art.manifest.assets.get(id) {
                let frame = frame_index(&animation.frames, draw.age, true, None);
                art.quad(
                    &painter,
                    id,
                    frame,
                    origin,
                    view.scale,
                    [1., 0., 0., 1., 0., 0.],
                    false,
                    0.7,
                );
            }
        } else if draw.id == "spell.glyph.zone" {
            // Composite one light red tint per glyph covering this navigation cell.
            let glyph_count = s
                .glyphs
                .iter()
                .filter(|glyph| draw.point.distance(glyph.position) <= d.spells["glyph"].radius)
                .count();
            let opacity =
                (255. * (1. - (1. - 48. / 255_f32).powf(glyph_count as f32))).round() as u8;
            let corners = [
                Vec2::new(0., 13.5),
                Vec2::new(26.5, 0.),
                Vec2::new(0., -13.5),
                Vec2::new(-26.5, 0.),
            ]
            .map(|offset| view.project(draw.point + offset))
            .to_vec();
            painter.add(egui::Shape::convex_polygon(
                corners,
                egui::Color32::from_rgba_unmultiplied(255, 30, 20, opacity),
                egui::Stroke::NONE,
            ));
            let a = &art.manifest.assets[&draw.id];
            art.quad(
                &painter,
                &draw.id,
                a.index(draw.age, None),
                origin,
                view.scale,
                [1., 0., 0., 1., 0., 0.],
                false,
                0.35,
            );
        } else if draw.scale == Vec2::ONE {
            art.sprite(
                &painter,
                &draw.id,
                draw.age,
                origin,
                view.scale,
                draw.flip,
                draw.angle,
                draw.window,
            );
        } else if let Some(a) = art.manifest.assets.get(&draw.id) {
            let (c, s) = (draw.angle.cos(), draw.angle.sin());
            art.quad(
                &painter,
                &draw.id,
                a.index(draw.age, draw.window),
                origin,
                view.scale,
                [
                    c * draw.scale.x,
                    -s * draw.scale.x,
                    s * draw.scale.y,
                    c * draw.scale.y,
                    0.,
                    0.,
                ],
                draw.flip.unwrap_or(a.flip_x),
                1.,
            );
        }
    }
    for (_, a, e) in actors {
        if a.hp <= 0. {
            continue;
        }
        let mut name = String::new();
        let boss = e.as_ref().is_some_and(|e| d.monsters[&e.kind].boss);
        if let Some(e) = e {
            name = d.monsters[&e.kind].name.clone();
            if let Some(dungeon) = s.dungeon.as_ref().and_then(|id| d.dungeons.get(id)) {
                let base_damage = (d.monsters[&e.kind].damage
                    * (1. + s.wave as f32 * dungeon.scaling.damage_per_wave))
                    .round();
                let bonus = e.damage - base_damage;
                if bonus > 0. {
                    name.push_str(&format!(" · Dégâts +{bonus:.0}"));
                }
            }
            if s.phase == Phase::Combat
                && d.monsters[&e.kind]
                    .invulnerability
                    .as_ref()
                    .is_some_and(|shield| shield.active(s.time - e.spawned))
            {
                name.push_str(" · Invulnérable");
            }
            if let Some(b) = &d.monsters[&e.kind].speed_burst {
                let age = s.time - e.spawned;
                if age >= b.interval && age % b.interval < b.duration {
                    name.push_str(&format!(" · Vitesse ×{:.0}", b.multiplier));
                }
            }
        } else if s.phase == Phase::Hub {
            continue;
        }
        let pos = view.project(a.position + Vec2::Y * if boss { 68. } else { 48. });
        let width = if boss { 100. } else { 60. };
        let rect = egui::Rect::from_center_size(pos, egui::vec2(width, 5.));
        if let Some((spell, until, _, _, _)) = active_buff.filter(|_| e.is_none()) {
            let badge =
                egui::Rect::from_center_size(pos - egui::vec2(0., 20.), egui::vec2(136., 24.));
            painter.rect_filled(
                badge,
                4.,
                egui::Color32::from_rgba_unmultiplied(50, 15, 20, 230),
            );
            art.in_rect(
                &painter,
                &format!("ui.spell.{spell}"),
                0.,
                egui::Rect::from_min_size(badge.min + egui::vec2(2., 2.), egui::vec2(20., 20.)),
                false,
            );
            painter.text(
                badge.left_center() + egui::vec2(26., 0.),
                egui::Align2::LEFT_CENTER,
                format!(
                    "{} · {:.0} s",
                    d.spells[spell].name,
                    (until - s.time).ceil()
                ),
                egui::FontId::proportional(12.),
                egui::Color32::from_rgb(255, 170, 160),
            );
        }
        painter.rect_filled(rect, 2., egui::Color32::from_black_alpha(190));
        let fill = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(
                rect.width() * (a.hp / a.max_hp).clamp(0., 1.),
                rect.height(),
            ),
        );
        painter.rect_filled(
            fill,
            2.,
            if e.is_some() {
                egui::Color32::from_rgb(98, 220, 121)
            } else {
                egui::Color32::from_rgb(98, 220, 121)
            },
        );
        if e.is_some() {
            painter.text(
                rect.right_center() + egui::vec2(4., 0.),
                egui::Align2::LEFT_CENTER,
                format!("{:.0}", a.hp),
                egui::FontId::proportional(9.),
                egui::Color32::WHITE,
            );
        }
        if !name.is_empty() {
            painter.text(
                pos - egui::vec2(0., 8.),
                egui::Align2::CENTER_BOTTOM,
                name,
                egui::FontId::proportional(if boss { 13. } else { 11. }),
                if boss {
                    egui::Color32::from_rgb(255, 205, 119)
                } else {
                    egui::Color32::WHITE
                },
            );
        }
    }
    for f in floating {
        let pos = view.project(
            f.position
                + Vec2::Y * (if f.kind == FloatKind::Loot { 72. } else { 48. } + f.age * 20.),
        );
        let font = egui::FontId::proportional(if f.kind == FloatKind::Damage {
            22.
        } else {
            12.
        });
        let tint = match f.kind {
            FloatKind::Damage => egui::Color32::from_rgb(255, 179, 156),
            FloatKind::Reward => egui::Color32::from_rgb(195, 255, 194),
            FloatKind::Loot => egui::Color32::from_rgb(255, 226, 171),
        };
        let galley = painter.layout_no_wrap(f.text.clone(), font, tint);
        let size = galley.size();
        let icon_width = if f.icon.is_some() { 24. } else { 0. };
        let left = pos.x - (size.x + icon_width) / 2.;
        if let Some(icon) = &f.icon {
            let rect =
                egui::Rect::from_min_size(egui::pos2(left, pos.y - 20.), egui::vec2(20., 20.));
            art.in_rect(&painter, icon, 0., rect, false);
        }
        let text_pos = egui::pos2(left + icon_width, pos.y - size.y);
        if f.kind == FloatKind::Damage {
            for offset in [
                egui::vec2(-1., -1.),
                egui::vec2(1., -1.),
                egui::vec2(-1., 1.),
                egui::vec2(1., 1.),
            ] {
                painter.galley_with_override_text_color(
                    text_pos + offset,
                    galley.clone(),
                    egui::Color32::from_black_alpha(230),
                );
            }
        }
        painter.galley(text_pos, galley, tint);
    }
}
