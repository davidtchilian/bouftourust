//! Épée de Iop : front de propagation invisible, animations fixes sur son passage.
use super::*;

const EFFECT_SPACING: f32 = 16.;

#[derive(Component, Clone)]
pub(super) struct SwordWave {
    pub start: Vec2,
    pub target: Vec2,
    pub amount: f32,
    distance: f32,
    next_effect: f32,
}
impl SwordWave {
    pub fn new(start: Vec2, target: Vec2, amount: f32) -> Self {
        Self {
            start,
            target,
            amount,
            distance: 0.,
            next_effect: 0.,
        }
    }
}

pub(super) fn update_sword_waves(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    m: &Manifest,
    dt: f32,
) {
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<SwordWave>>()
        .iter(world)
        .collect();
    let spell = &d.spells["sword"];
    for id in ids {
        if s.phase != Phase::Combat {
            world.despawn(id);
            continue;
        }
        let mut wave = world.get::<SwordWave>(id).unwrap().clone();
        let length = wave.start.distance(wave.target);
        let direction = (wave.target - wave.start).normalize_or_zero();
        let old = wave.start + direction * wave.distance;
        let distance = (wave.distance + spell.speed * dt).min(length);
        let next = wave.start + direction * distance;
        let blocked = !s.navigation.clear(old, next);
        if !blocked {
            wave.distance = distance;
        }
        let arrived = blocked || wave.distance >= length;
        // Une animation est créée à chaque intervalle franchi. Aucune de ces
        // entités ne se déplace, même si le personnage bouge après le lancer.
        while wave.next_effect <= wave.distance && (!arrived || wave.next_effect < wave.distance) {
            effect(
                world,
                m,
                &spell.effect,
                wave.start + direction * wave.next_effect,
                0.,
                false,
            );
            wave.next_effect += EFFECT_SPACING;
        }
        if arrived {
            let impact = wave.start + direction * wave.distance;
            effect(world, m, &spell.effect, impact, 0., false);
            for (enemy, actor, _) in living(world) {
                if actor.position.distance(impact) <= spell.radius {
                    hit(world, s, p, d, enemy, wave.amount);
                }
            }
            world.despawn(id);
        } else {
            *world.get_mut::<SwordWave>(id).unwrap() = wave;
        }
    }
}
