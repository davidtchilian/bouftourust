use bevy::prelude::*;
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub hp: f32,
    pub power: f32,
    #[serde(default)]
    pub cooldown_reduction: f32,
    #[serde(default)]
    pub movement_speed: f32,
}
impl Stats {
    pub fn add(&mut self, s: &Self) {
        self.hp += s.hp;
        self.power += s.power;
        self.cooldown_reduction += s.cooldown_reduction;
        self.movement_speed += s.movement_speed;
    }
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub name: String,
    pub category: String,
    pub stats: Stats,
    pub set: String,
    pub icon: String,
    pub stat_limits: Option<Limits>,
}
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    #[serde(default)]
    pub movement_speed: f32,
    #[serde(default)]
    pub cooldown_reduction: f32,
}
#[derive(Clone, Deserialize)]
pub struct Set {
    pub name: String,
    pub thresholds: BTreeMap<u32, Stats>,
}
#[derive(Clone, Deserialize)]
pub struct Drop {
    pub item: String,
    pub weight: f32,
}
#[derive(Clone, Deserialize)]
pub struct BonusLoot {
    pub chance: f32,
    pub table: Vec<Drop>,
}
#[derive(Clone, Deserialize)]
pub struct Burst {
    pub interval: f32,
    pub duration: f32,
    pub multiplier: f32,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ranged {
    pub range: f32,
    pub speed: f32,
    pub radius: f32,
    pub windup: f32,
    pub animation_duration: f32,
    pub asset: String,
    pub animation: String,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monster {
    pub name: String,
    pub sprite: String,
    pub hp: f32,
    pub speed: f32,
    pub detect: f32,
    pub range: f32,
    pub damage: f32,
    pub interval: f32,
    pub xp: u32,
    pub kamas: u64,
    #[serde(default)]
    pub boss: bool,
    pub loot: Vec<Drop>,
    pub bonus_loot: Option<BonusLoot>,
    pub speed_burst: Option<Burst>,
    pub ranged_attack: Option<Ranged>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dungeon {
    pub id: String,
    pub name: String,
    pub map: String,
    pub reward_set: String,
    pub tier: u32,
    pub recommended_gear: String,
    pub normal_waves: Vec<Vec<String>>,
    pub boss_wave: Vec<String>,
    pub description: String,
    pub scaling: Scaling,
    pub loot: Loot,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scaling {
    pub hp_per_wave: f32,
    pub damage_per_wave: f32,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Loot {
    pub normal_chance: f32,
    pub boss_chance: f32,
}
#[derive(Clone, Deserialize)]
pub struct Spell {
    pub name: String,
    pub coefficient: f32,
    pub cooldown: f32,
    pub range: Option<f32>,
    pub speed: f32,
    pub radius: f32,
    pub kind: String,
    pub effect: String,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub id: String,
    pub name: String,
    pub sprite: String,
    pub base_damage: f32,
    pub speed: f32,
    pub abilities: Vec<String>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub max_level: u32,
    pub starting_hp: f32,
    pub hp_per_level: f32,
    pub power_per_level: f32,
    pub wave_delay: f32,
    pub cast_lock: f32,
    pub cast_animation: f32,
    pub enemy_attack_animation: f32,
    pub enemy_hit_animation: f32,
    pub enemy_death_animation: f32,
    pub buff_multiplier: f32,
    pub buff_duration: f32,
    pub max_cooldown_reduction: f32,
    pub max_movement_speed_bonus: f32,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassBalance {
    pub drain_ratio: f32,
    pub chastisement_duration: f32,
    pub chastisement_power_per_damage: f32,
    pub low_hp_maximum_bonus: f32,
    pub low_hp_threshold: f32,
    pub push_distance: f32,
    pub wrath_growth: f32,
    pub truce_duration: f32,
    pub glyph_interval: f32,
    pub cone_cosine: f32,
    pub target_radius: f32,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DungeonMode {
    pub label: String,
    pub monster_count: usize,
    pub hp: f32,
    pub complete_set: bool,
}
#[derive(Resource, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub balance: Balance,
    pub class_balance: ClassBalance,
    pub classes: BTreeMap<String, Class>,
    pub spells: BTreeMap<String, Spell>,
    pub monsters: BTreeMap<String, Monster>,
    pub dungeons: BTreeMap<String, Dungeon>,
    pub dungeon_modes: BTreeMap<String, DungeonMode>,
    pub items: BTreeMap<String, Item>,
    pub sets: BTreeMap<String, Set>,
    pub slots: Vec<String>,
    pub slot_names: BTreeMap<String, String>,
    pub resale_prices: BTreeMap<String, u64>,
    pub spell_descriptions: BTreeMap<String, String>,
}
impl GameData {
    pub fn load(root: &Path) -> Result<Self, String> {
        let p = root.join("data/game.json");
        serde_json::from_slice(&std::fs::read(&p).map_err(|e| format!("{} : {e}", p.display()))?)
            .map_err(|e| e.to_string())
    }
}
pub fn xp_for_level(level: u32) -> u32 {
    (25.0 * (level as f64).powf(1.4)).floor() as u32
}
pub fn damage(base: f32, coefficient: f32, power: f32, buff: f32) -> f32 {
    (base * coefficient * (1.0 + power / 100.0) * buff)
        .floor()
        .max(0.0)
}
