use bevy::prelude::*;
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub path: String,
    pub duration_ms: f32,
    pub region: Option<[u32; 6]>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animation {
    pub frames: Vec<Frame>,
    pub pivot: [f32; 2],
    pub size: [f32; 2],
    pub offset: [f32; 2],
    #[serde(rename = "loop")]
    pub looping: bool,
    #[serde(default)]
    pub flip_x: bool,
}
impl Animation {
    pub fn index(&self, seconds: f32, window: Option<f32>) -> usize {
        frame_index(&self.frames, seconds, self.looping, window)
    }
}
pub fn frame_index(frames: &[Frame], seconds: f32, looping: bool, window: Option<f32>) -> usize {
    let total: f32 = frames.iter().map(|f| f.duration_ms).sum();
    let mut t = seconds.max(0.) * 1000.;
    if let Some(w) = window {
        t = seconds.max(0.) / w * total;
    }
    if looping && window.is_none() {
        t %= total;
    } else {
        t = t.min((total - 0.001).max(0.));
    }
    frames
        .iter()
        .position(|f| {
            t -= f.duration_ms;
            t < 0.
        })
        .unwrap_or(frames.len() - 1)
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub asset_id: String,
    pub position: [f32; 2],
    pub order: f32,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default = "unit_scale")]
    pub scale: [f32; 2],
}
#[derive(Clone, Deserialize)]
pub struct Cell {
    pub id: usize,
    pub position: [f32; 2],
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub bounds: [f32; 4],
    pub spawn: [f32; 2],
    pub camera_height: f32,
    #[serde(default)]
    pub layers: Vec<Layer>,
    pub walkable: Vec<Cell>,
    #[serde(default)]
    pub enemy_spawns: Vec<[f32; 2]>,
    pub background_asset_id: Option<String>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(default)]
    pub masked_assets: BTreeMap<String, String>,
    pub asset_id: Option<String>,
    pub slot: Option<String>,
    pub label: Option<String>,
    pub mask_asset_id: Option<String>,
    #[serde(default = "full_opacity")]
    pub opacity: f32,
    pub matrix: [f32; 6],
    #[serde(default)]
    pub hide_with_hat: bool,
    #[serde(default)]
    pub show_with_hat: bool,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoseFrame {
    pub duration_ms: f32,
    pub layers: Vec<Part>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pose {
    #[serde(rename = "loop")]
    pub looping: bool,
    pub flip_x: bool,
    pub frames: Vec<PoseFrame>,
}
#[derive(Clone, Deserialize)]
pub struct Accessory {
    pub slot: String,
    pub views: BTreeMap<String, String>,
}
#[derive(Clone, Deserialize)]
pub struct Composition {
    pub actors: BTreeMap<String, Pose>,
    pub accessories: BTreeMap<String, Accessory>,
}
#[derive(Resource, Clone, Deserialize)]
pub struct Manifest {
    pub assets: BTreeMap<String, Animation>,
    pub hub: Map,
    pub maps: BTreeMap<String, Map>,
    pub composition: Composition,
}
impl Manifest {
    pub fn load(root: &Path) -> Result<Self, String> {
        let p = root.join("manifest.json");
        serde_json::from_slice(
            &std::fs::read(&p)
                .map_err(|e| format!("{} : {e}. Voir docs/ASSETS.md", p.display()))?,
        )
        .map_err(|e| e.to_string())
    }
    pub fn validate(&self, data: &crate::data::GameData) -> Result<(), String> {
        let mut required = Vec::new();
        for class in data.classes.values() {
            for dir in crate::simulation::DIRECTIONS {
                for action in ["idle", "run"] {
                    let id = format!("{}.{}.{dir}", class.id, action);
                    if !self.composition.actors.contains_key(&id) {
                        return Err(format!("Composition absente : {id}"));
                    }
                    required.push(id);
                }
            }
            for side in ["L", "R"] {
                for action in ["cast", "buff", "hit", "death"] {
                    let id = format!("{}.{}.{side}", class.id, action);
                    if !self.composition.actors.contains_key(&id) {
                        return Err(format!("Composition absente : {id}"));
                    }
                    required.push(id);
                }
            }
            for id in &class.abilities {
                required.push(format!("ui.spell.{id}"));
                let effect = &data.spells[id].effect;
                if !effect.is_empty() {
                    required.push(effect.clone());
                }
            }
        }
        for monster in data.monsters.values() {
            for action in ["idle", "run", "attack", "hit", "death"] {
                for side in ["R", "L"] {
                    required.push(format!("mob.{}.{}.{side}", monster.sprite, action));
                }
            }
            if let Some(r) = &monster.ranged_attack {
                required.push(r.asset.clone());
                for side in ["R", "L"] {
                    required.push(format!("{}.{side}", r.animation));
                }
            }
        }
        for item in data.items.values() {
            required.push(item.icon.clone());
        }
        for dungeon in data.dungeons.values() {
            if !self.maps.contains_key(&dungeon.map) {
                return Err(format!("Carte absente : {}", dungeon.map));
            }
        }
        for map in std::iter::once(&self.hub).chain(self.maps.values()) {
            if map.walkable.is_empty() {
                return Err("Carte sans navigation".into());
            }
            for l in &map.layers {
                required.push(l.asset_id.clone());
            }
            if let Some(id) = &map.background_asset_id {
                required.push(id.clone());
            }
        }
        for id in required {
            if !self.assets.contains_key(&id) {
                return Err(format!(
                    "Asset Remastered absent : {id}. Voir docs/ASSETS.md"
                ));
            }
        }
        Ok(())
    }
    pub fn map(&self, id: &str) -> &Map {
        if id == "hub" {
            &self.hub
        } else {
            &self.maps[id]
        }
    }
}

fn full_opacity() -> f32 {
    1.
}

fn unit_scale() -> [f32; 2] {
    [1., 1.]
}
