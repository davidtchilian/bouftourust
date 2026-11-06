use crate::data::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub uid: String,
    pub item: String,
}
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub version: u32,
    pub class_id: String,
    pub level: u32,
    pub xp: u32,
    pub kamas: u64,
    pub inventory: Vec<Instance>,
    pub equipment: BTreeMap<String, String>,
    pub completed: Vec<String>,
    pub serial: u64,
    pub help_seen: bool,
    pub reduced_motion: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            version: 1,
            class_id: "cra".into(),
            level: 1,
            xp: 0,
            kamas: 0,
            inventory: vec![],
            equipment: BTreeMap::new(),
            completed: vec![],
            serial: 0,
            help_seen: false,
            reduced_motion: false,
        }
    }
}
impl Profile {
    pub fn worn(&self, slot: &str) -> Option<&str> {
        let uid = self.equipment.get(slot)?;
        self.inventory
            .iter()
            .find(|i| &i.uid == uid)
            .map(|i| i.item.as_str())
    }
    pub fn limits(&self, data: &GameData) -> Limits {
        let mut l = Limits {
            movement_speed: data.balance.max_movement_speed_bonus,
            cooldown_reduction: data.balance.max_cooldown_reduction,
        };
        for slot in &data.slots {
            if let Some(s) = self
                .worn(slot)
                .and_then(|id| data.items.get(id))
                .and_then(|i| i.stat_limits.as_ref())
            {
                l.movement_speed = l.movement_speed.max(s.movement_speed);
                l.cooldown_reduction = l.cooldown_reduction.max(s.cooldown_reduction);
            }
        }
        l
    }
    pub fn set_counts(&self, data: &GameData) -> BTreeMap<String, BTreeSet<String>> {
        let mut sets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for slot in &data.slots {
            if let Some(item) = self.worn(slot).and_then(|id| data.items.get(id)) {
                sets.entry(item.set.clone())
                    .or_default()
                    .insert(item.id.clone());
            }
        }
        sets
    }
    pub fn stats(&self, data: &GameData) -> Stats {
        let mut stats = Stats {
            hp: data.balance.starting_hp + (self.level - 1) as f32 * data.balance.hp_per_level,
            power: (self.level - 1) as f32 * data.balance.power_per_level,
            ..default()
        };
        for slot in &data.slots {
            if let Some(item) = self.worn(slot).and_then(|id| data.items.get(id)) {
                stats.add(&item.stats);
            }
        }
        for (id, ids) in self.set_counts(data) {
            if let Some(set) = data.sets.get(&id) {
                if let Some((_, s)) = set.thresholds.range(..=ids.len() as u32).next_back() {
                    stats.add(s);
                }
            }
        }
        let limits = self.limits(data);
        stats.cooldown_reduction = stats
            .cooldown_reduction
            .clamp(0., limits.cooldown_reduction);
        stats.movement_speed = stats.movement_speed.clamp(-50., limits.movement_speed);
        stats
    }
    pub fn award_xp(&mut self, amount: u32, data: &GameData) -> f32 {
        let old = self.level;
        self.xp = self.xp.saturating_add(amount);
        while self.level < data.balance.max_level && self.xp >= xp_for_level(self.level) {
            self.xp -= xp_for_level(self.level);
            self.level += 1;
        }
        if self.level == data.balance.max_level {
            self.xp = 0;
        }
        (self.level - old) as f32 * data.balance.hp_per_level
    }
    pub fn give(&mut self, item: &str) {
        loop {
            self.serial += 1;
            if !self
                .inventory
                .iter()
                .any(|i| i.uid == format!("local-{}", self.serial))
            {
                break;
            }
        }
        self.inventory.push(Instance {
            uid: format!("local-{}", self.serial),
            item: item.into(),
        });
    }
    pub fn equip(&mut self, uid: &str, slot: Option<&str>, data: &GameData) -> bool {
        let Some(instance) = self.inventory.iter().find(|i| i.uid == uid) else {
            return false;
        };
        let Some(item) = data.items.get(&instance.item) else {
            return false;
        };
        let target = slot.map(str::to_owned).unwrap_or_else(|| {
            if item.category == "ring" {
                if self.equipment.contains_key("ring1") {
                    "ring2"
                } else {
                    "ring1"
                }
                .into()
            } else {
                item.category.clone()
            }
        });
        if item.category == "ring" {
            if target != "ring1" && target != "ring2" {
                return false;
            }
        } else if item.category != target {
            return false;
        }
        self.equipment.retain(|_, v| v != uid);
        self.equipment.insert(target, uid.into());
        true
    }
    pub fn sale_plan(&self, data: &GameData) -> (BTreeSet<String>, u64) {
        let worn: BTreeSet<_> = self.equipment.values().collect();
        let worn_types: BTreeSet<_> = self
            .inventory
            .iter()
            .filter(|i| worn.contains(&i.uid))
            .map(|i| &i.item)
            .collect();
        let mut seen = BTreeSet::new();
        let mut sold = BTreeSet::new();
        let mut value = 0;
        for i in &self.inventory {
            if worn.contains(&i.uid) {
                continue;
            }
            if !worn_types.contains(&i.item) && seen.insert(&i.item) {
                continue;
            }
            if let Some(item) = data.items.get(&i.item) {
                sold.insert(i.uid.clone());
                value += data.resale_prices[&item.category];
            }
        }
        (sold, value)
    }
    pub fn sell(&mut self, data: &GameData) -> (usize, u64) {
        let (sold, value) = self.sale_plan(data);
        self.inventory.retain(|i| !sold.contains(&i.uid));
        self.kamas += value;
        (sold.len(), value)
    }
    pub fn validate(&self, data: &GameData) -> Result<(), String> {
        if self.version != 1
            || self.level < 1
            || self.level > data.balance.max_level
            || !data.classes.contains_key(&self.class_id)
        {
            return Err("Version, niveau ou classe de sauvegarde invalide".into());
        }
        if self.level < data.balance.max_level && self.xp >= xp_for_level(self.level) {
            return Err("XP incohérente avec le niveau".into());
        }
        if self
            .completed
            .iter()
            .any(|id| !data.dungeons.contains_key(id))
        {
            return Err("Donjon accompli inconnu".into());
        }
        let mut uids = BTreeSet::new();
        for i in &self.inventory {
            if !data.items.contains_key(&i.item) || !uids.insert(&i.uid) {
                return Err("Objet inconnu ou identifiant en double".into());
            }
        }
        let mut worn = BTreeSet::new();
        for (slot, uid) in &self.equipment {
            let Some(i) = self.inventory.iter().find(|i| &i.uid == uid) else {
                return Err("Équipement sans objet possédé".into());
            };
            let category = &data.items[&i.item].category;
            if !data.slots.contains(slot)
                || !worn.insert(uid)
                || (category == "ring" && slot != "ring1" && slot != "ring2")
                || (category != "ring" && category != slot)
            {
                return Err("Emplacement invalide".into());
            }
        }
        Ok(())
    }
}
#[derive(Resource)]
pub struct SaveStore {
    pub path: PathBuf,
    pub dirty: bool,
    pub error: Option<String>,
}
impl SaveStore {
    pub fn local() -> Self {
        let path = std::env::var_os("BOUFTOURUSH_SAVE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                directories::ProjectDirs::from("fr", "BouftouRush", "BouftouRush")
                    .expect("Dossier utilisateur")
                    .data_local_dir()
                    .join("profile.json")
            });
        Self {
            path,
            dirty: false,
            error: None,
        }
    }
    pub fn load(&self, data: &GameData) -> Result<Profile, String> {
        if !self.path.exists() {
            return Ok(Profile::default());
        }
        let p: Profile =
            serde_json::from_slice(&std::fs::read(&self.path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("Sauvegarde illisible : {e}"))?;
        p.validate(data)?;
        Ok(p)
    }
    /// Le fichier précédent reste disponible si le processus s'interrompt pendant l'écriture.
    pub fn save(&mut self, profile: &Profile) -> Result<(), String> {
        let parent = self.path.parent().ok_or("Chemin sans dossier")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("tmp");
        let bytes = serde_json::to_vec_pretty(profile).map_err(|e| e.to_string())?;
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
            f.write_all(&bytes)
                .and_then(|_| f.sync_all())
                .map_err(|e| e.to_string())?;
        }
        if self.path.exists() {
            std::fs::copy(&self.path, self.path.with_extension("bak"))
                .map_err(|e| e.to_string())?;
        }
        std::fs::rename(&tmp, &self.path).map_err(|e| e.to_string())?;
        self.dirty = false;
        self.error = None;
        Ok(())
    }
}

/// Écrit les snapshots hors du thread de jeu. File bornée : un snapshot en attente.
#[derive(Resource)]
pub struct SaveWorker {
    sender: Option<std::sync::mpsc::SyncSender<Profile>>,
    results: std::sync::Mutex<std::sync::mpsc::Receiver<Result<(), String>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl SaveWorker {
    pub fn new(path: PathBuf) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<Profile>(1);
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut store = SaveStore {
                path,
                dirty: false,
                error: None,
            };
            while let Ok(profile) = rx.recv() {
                if result_tx.send(store.save(&profile)).is_err() {
                    break;
                }
            }
        });
        Self {
            sender: Some(tx),
            results: std::sync::Mutex::new(result_rx),
            thread: Some(thread),
        }
    }
    pub fn enqueue(&self, p: &Profile) -> Result<bool, String> {
        match self
            .sender
            .as_ref()
            .ok_or("Writer arrêté")?
            .try_send(p.clone())
        {
            Ok(()) => Ok(true),
            Err(std::sync::mpsc::TrySendError::Full(_)) => Ok(false),
            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                Err("Writer de sauvegarde interrompu".into())
            }
        }
    }
    pub fn poll(&self) -> Option<Result<(), String>> {
        let rx = self.results.lock().ok()?;
        let mut last = None;
        while let Ok(result) = rx.try_recv() {
            last = Some(result);
        }
        last
    }
}
impl std::ops::Drop for SaveWorker {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
