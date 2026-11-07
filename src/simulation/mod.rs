//! Simulation ECS déterministe. Aucun accès au GPU ou au réseau.
use crate::{assets::*, data::*, navigation::*, profile::*};
mod actions;
use actions::*;
mod movement;
use movement::*;
mod bosses;
mod combat;
use combat::*;
mod sram;
mod sword;
pub use sram::Trap;
use sword::*;
mod enemies;
use enemies::*;
mod waves;
use waves::*;
mod visuals;
pub use enemies::burst_seconds;
use visuals::*;

use bevy::prelude::*;
use std::collections::{BTreeSet, VecDeque};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Hub,
    Combat,
    Victory,
    Death,
}
#[derive(Component, Clone)]
pub struct Actor {
    pub position: Vec2,
    pub direction: usize,
    pub action: &'static str,
    pub animation: f32,
    pub until: f32,
    pub route: VecDeque<Vec2>,
    pub hp: f32,
    pub max_hp: f32,
}
impl Actor {
    pub fn new(p: Vec2, hp: f32) -> Self {
        Self {
            position: p,
            direction: 7,
            action: "idle",
            animation: 0.,
            until: 0.,
            route: default(),
            hp,
            max_hp: hp,
        }
    }
    pub fn action(&mut self, s: &'static str) {
        if self.action != s {
            self.action = s;
            self.animation = 0.;
        }
    }
}
#[derive(Component)]
pub struct Player;
#[derive(Component, Default)]
pub struct AllyAttackBonus(pub f32);
#[derive(Component, Clone)]
pub struct Enemy {
    pub kind: String,
    pub damage: f32,
    pub spawned: f32,
    pub attack_at: f32,
    pub path_at: f32,
    pub dead_at: f32,
    pub ranged_target: Option<(Vec2, f32)>,
    pub ranged_until: f32,
}
#[derive(Component, Clone)]
pub struct Shot {
    pub spell: String,
    pub position: Vec2,
    pub start: Vec2,
    pub target: Vec2,
    pub amount: f32,
    pub speed: f32,
    pub radius: f32,
    pub hit: BTreeSet<Entity>,
    pub age: f32,
    pub hostile: bool,
    pub asset: String,
}
#[derive(Component, Clone)]
pub struct Effect {
    pub asset: String,
    pub position: Vec2,
    pub age: f32,
    pub ttl: f32,
    pub angle: f32,
    pub follow: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FloatKind {
    Damage,
    Reward,
    Loot,
}
#[derive(Component, Clone)]
pub struct Floating {
    pub position: Vec2,
    pub text: String,
    pub age: f32,
    pub kind: FloatKind,
    pub icon: Option<String>,
}
#[derive(Clone)]
pub struct Glyph {
    pub position: Vec2,
    pub next_tick: f32,
}
#[derive(Resource)]
pub struct Session {
    pub player: Entity,
    pub phase: Phase,
    pub map_id: String,
    pub revision: u64,
    pub dungeon: Option<String>,
    pub extreme: bool,
    pub wave: i32,
    pub waiting: bool,
    pub delay: f32,
    pub time: f32,
    pub paused: bool,
    pub loading: bool,
    pub cooldowns: [f32; 4],
    pub buff_until: f32,
    pub cast_until: f32,
    pub chastisement_until: f32,
    pub combat_power: f32,
    pub wrath: f32,
    pub truce_until: f32,
    pub glyphs: Vec<Glyph>,
    pub traps: Vec<sram::Trap>,
    pub invisible_until: f32,
    pub last_known_player: Vec2,
    pub navigation: Navigation,
    pub destination: Option<Vec2>,
    pub log: VecDeque<String>,
    pub random: u32,
    pub kills: u32,
}
impl Session {
    pub fn message(&mut self, s: impl Into<String>) {
        self.log.push_front(s.into());
        self.log.truncate(8);
    }
    pub fn random(&mut self) -> f32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 17;
        self.random ^= self.random << 5;
        self.random as f64 as f32 / 4294967296.
    }
    pub fn map<'a>(&self, m: &'a Manifest) -> &'a Map {
        m.map(&self.map_id)
    }
}
#[derive(Clone)]
pub enum Action {
    Move(Vec2),
    Stop,
    Cast(usize, Vec2),
    Enter(String, bool),
    Hub,
    Class(String),
    Equip(String, Option<String>),
    Unequip(String),
    Sell,
    Fuse(String, u8),
    Preferences {
        help_seen: bool,
        reduced_motion: bool,
    },
}
#[derive(Resource, Default)]
pub struct Actions(pub Vec<Action>);
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>()
            .add_systems(Startup, setup)
            .add_systems(FixedUpdate, step);
    }
}
fn setup(world: &mut World) {
    let data = world.resource::<GameData>();
    let p = world.resource::<Profile>();
    let hp = p.stats(data).hp;
    let map = world.resource::<Manifest>().hub.clone();
    let player = world.spawn((Player, Actor::new(map.spawn.into(), hp))).id();
    let mut s = Session {
        player,
        phase: Phase::Hub,
        map_id: "hub".into(),
        revision: 0,
        dungeon: None,
        extreme: false,
        wave: -1,
        waiting: true,
        delay: 1.8,
        time: 0.,
        paused: false,
        loading: false,
        cooldowns: [0.; 4],
        buff_until: 0.,
        cast_until: 0.,
        chastisement_until: 0.,
        combat_power: 0.,
        wrath: 1.,
        truce_until: 0.,
        glyphs: vec![],
        traps: vec![],
        invisible_until: 0.,
        last_known_player: map.spawn.into(),
        navigation: Navigation::new(&map.walkable),
        destination: None,
        log: default(),
        random: 42,
        kills: 0,
    };
    s.message("Bienvenue à Astrub. Clic droit : déplacement · AZER : sorts · I : inventaire.");
    world.insert_resource(s);
}
pub fn direction(delta: Vec2) -> usize {
    ((delta.y.atan2(delta.x) / std::f32::consts::FRAC_PI_4).round() as i32).rem_euclid(8) as usize
}
pub const DIRECTIONS: [&str; 8] = [
    "east",
    "northeast",
    "north",
    "northwest",
    "west",
    "southwest",
    "south",
    "southeast",
];
pub fn facing(dir: usize) -> (&'static str, bool) {
    let back = (1..=3).contains(&dir);
    (
        if back { "L" } else { "R" },
        if back { dir == 1 } else { dir == 4 || dir == 5 },
    )
}
fn living(world: &mut World) -> Vec<(Entity, Actor, Enemy)> {
    world
        .query::<(Entity, &Actor, &Enemy)>()
        .iter(world)
        .filter(|(_, a, _)| a.hp > 0.)
        .map(|(e, a, m)| (e, a.clone(), m.clone()))
        .collect()
}
fn player_movement(world: &mut World, s: &Session, p: &Profile, d: &GameData, dt: f32) {
    let mut a = world.get_mut::<Actor>(s.player).unwrap();
    a.max_hp = p.stats(d).hp;
    a.animation += dt;
    if matches!(s.phase, Phase::Death | Phase::Victory) {
        return;
    }
    let movement_dt = dt.min((s.time - s.cast_until).max(0.));
    let moving = movement_dt > 0.
        && move_actor(
            &mut a,
            &s.navigation,
            d.classes[&p.class_id].speed * (1. + p.stats(d).movement_speed / 100.),
            movement_dt,
        );
    if s.time >= a.until {
        a.action(if moving { "run" } else { "idle" });
    }
}
fn step(world: &mut World) {
    let dt = world.resource::<Time<Fixed>>().delta_secs();
    let mut s = world.remove_resource::<Session>().unwrap();
    let mut p = world.remove_resource::<Profile>().unwrap();
    let d = world.remove_resource::<GameData>().unwrap();
    let m = world.remove_resource::<Manifest>().unwrap();
    process_actions(world, &mut s, &mut p, &d, &m);
    if !s.paused && !s.loading {
        s.time += dt;
        player_movement(world, &s, &p, &d, dt);
        if s.phase == Phase::Hub {
            update_shots(world, &mut s, &mut p, &d, &m, dt);
        }
        if s.phase == Phase::Combat {
            sram::trigger_traps(world, &mut s, &mut p, &d, &m, None);
            glyphs(world, &mut s, &mut p, &d);
            update_sword_waves(world, &mut s, &mut p, &d, &m, dt);
            update_shots(world, &mut s, &mut p, &d, &m, dt);
            if s.phase == Phase::Combat {
                bosses::boss_abilities(world, &mut s, &d);
                enemies(world, &mut s, &mut p, &d, &m, dt);
                waves(world, &mut s, &mut p, &d, &m, dt);
            }
        }
        visuals_age(world, &s, dt);
    }
    if let Some(result) = world
        .get_resource::<SaveWorker>()
        .and_then(SaveWorker::poll)
    {
        world.resource_mut::<SaveStore>().error = result.err();
    }
    if world.resource::<SaveStore>().dirty {
        let result = if let Some(writer) = world.get_resource::<SaveWorker>() {
            writer.enqueue(&p)
        } else {
            world.resource_mut::<SaveStore>().save(&p).map(|_| true)
        };
        match result {
            Ok(true) => world.resource_mut::<SaveStore>().dirty = false,
            Ok(false) => {}
            Err(err) => {
                world.resource_mut::<SaveStore>().error = Some(err.clone());
                s.message(format!("Échec de sauvegarde : {err}"));
                world.resource_mut::<SaveStore>().dirty = false;
            }
        }
    }
    world.insert_resource(s);
    world.insert_resource(p);
    world.insert_resource(d);
    world.insert_resource(m);
}
