use super::*;

pub(super) fn waves(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    m: &Manifest,
    dt: f32,
) {
    if s.phase != Phase::Combat {
        return;
    }
    let alive = world
        .query::<&Actor>()
        .iter(world)
        .filter(|a| a.hp > 0.)
        .count()
        .saturating_sub(1);
    let dungeon = &d.dungeons[s.dungeon.as_ref().unwrap()];
    let total = dungeon.normal_waves.len() + 1;
    let mode = &d.dungeon_modes[if s.extreme { "extreme" } else { "normal" }];
    if !s.waiting {
        if alive > 0 {
            return;
        }
        if s.wave as usize == total - 1 {
            s.phase = Phase::Victory;
            s.glyphs.clear();
            s.traps.clear();
            s.invisible_until = 0.;
            s.combat_power = 0.;
            s.truce_until = 0.;
            let mut a = world.get_mut::<Actor>(s.player).unwrap();
            a.route.clear();
            a.action("idle");
            if !p.completed.contains(&dungeon.id) {
                p.completed.push(dungeon.id.clone());
            }
            if mode.complete_set {
                for item in d.items.values().filter(|i| i.set == dungeon.reward_set) {
                    p.give(&item.id);
                }
                s.message(format!(
                    "Victoire Extrême : {} complète !",
                    d.sets[&dungeon.reward_set].name
                ));
            }
            s.message("Donjon terminé ! Récompenses sauvegardées.");
            let ids: Vec<_> = world
                .query_filtered::<Entity, With<Shot>>()
                .iter(world)
                .collect();
            for e in ids {
                world.despawn(e);
            }
            world.resource_mut::<SaveStore>().dirty = true;
            return;
        }
        s.waiting = true;
        s.delay = d.balance.wave_delay;
    }
    s.delay -= dt;
    if s.delay > 0. {
        return;
    }
    s.wave += 1;
    s.waiting = false;
    let wave = if s.wave as usize == total - 1 {
        &dungeon.boss_wave
    } else {
        &dungeon.normal_waves[s.wave as usize]
    };
    let map = s.map(m);
    let mut occupied = BTreeSet::new();
    let player = world.get::<Actor>(s.player).unwrap().position;
    let multiplier = mode.monster_count;
    let mut index = 0;
    let corpses: Vec<_> = world
        .query::<(Entity, &Actor, &Enemy)>()
        .iter(world)
        .filter(|(_, a, _)| a.hp <= 0.)
        .map(|(e, _, _)| e)
        .collect();
    for e in corpses {
        world.despawn(e);
    }
    for kind in wave {
        for _ in 0..multiplier {
            let anchor: Vec2 = map
                .enemy_spawns
                .get(index % map.enemy_spawns.len().max(1))
                .copied()
                .unwrap_or(map.spawn)
                .into();
            let mut candidates: Vec<_> = s
                .navigation
                .cells
                .iter()
                .enumerate()
                .filter(|(i, c)| {
                    !occupied.contains(i)
                        && Vec2::from(c.position).distance(map.spawn.into()) >= 70.
                        && Vec2::from(c.position).distance(player) >= 70.
                })
                .collect();
            candidates.sort_by(|(_, a), (_, b)| {
                Vec2::from(a.position)
                    .distance_squared(anchor)
                    .total_cmp(&Vec2::from(b.position).distance_squared(anchor))
            });
            let cell = candidates
                .iter()
                .find(|(_, c)| s.navigation.clear(anchor, c.position.into()))
                .copied()
                .or_else(|| candidates.first().copied());
            let pos = if let Some((i, c)) = cell {
                occupied.insert(i);
                c.position.into()
            } else {
                anchor
            };
            let def = &d.monsters[kind];
            let hp =
                (def.hp * (1. + s.wave as f32 * dungeon.scaling.hp_per_wave) * mode.hp).round();
            let amount =
                (def.damage * (1. + s.wave as f32 * dungeon.scaling.damage_per_wave)).round();
            world.spawn((
                Actor::new(pos, hp),
                Enemy {
                    kind: kind.clone(),
                    damage: amount,
                    spawned: s.time,
                    attack_at: s.time + 0.6 + index as f32 * 0.13,
                    path_at: 0.,
                    dead_at: 0.,
                    ranged_target: None,
                    ranged_until: 0.,
                },
            ));
            index += 1;
        }
    }
    s.message(format!(
        "Vague {}/{}{}",
        s.wave + 1,
        total,
        if s.wave as usize == total - 1 {
            " · Boss"
        } else {
            ""
        }
    ));
}
