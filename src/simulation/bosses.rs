use super::*;

#[derive(Component)]
struct BossTimers {
    heal: f32,
    summon: f32,
}

pub(super) fn boss_abilities(world: &mut World, s: &mut Session, d: &GameData) {
    if s.phase != Phase::Combat || s.paused || s.loading {
        return;
    }
    for (id, actor, enemy) in living(world) {
        let def = &d.monsters[&enemy.kind];
        if def.ally_heal.is_none() && def.summon.is_none() {
            continue;
        }
        if world.get::<BossTimers>(id).is_none() {
            world.entity_mut(id).insert(BossTimers {
                heal: enemy.spawned + def.ally_heal.as_ref().map_or(f32::INFINITY, |h| h.interval),
                summon: enemy.spawned + def.summon.as_ref().map_or(f32::INFINITY, |h| h.interval),
            });
        }
        if let Some(heal) = &def.ally_heal {
            while s.time >= world.get::<BossTimers>(id).unwrap().heal {
                world.get_mut::<BossTimers>(id).unwrap().heal += heal.interval;
                for (ally, a, _) in living(world) {
                    if ally == id {
                        continue;
                    }
                    let restored = heal.amount.min((a.max_hp - a.hp).max(0.));
                    if restored > 0. {
                        world.get_mut::<Actor>(ally).unwrap().hp += restored;
                        float(
                            world,
                            format!("+{restored:.0} PV"),
                            a.position,
                            FloatKind::Reward,
                            None,
                        );
                    }
                }
                s.message(format!(
                    "{} soigne ses alliés (+{:.0} PV).",
                    def.name, heal.amount
                ));
            }
        }
        if let Some(summon) = &def.summon {
            while s.time >= world.get::<BossTimers>(id).unwrap().summon {
                world.get_mut::<BossTimers>(id).unwrap().summon += summon.interval;
                let mut spawned = 0;
                for _ in 0..summon.count {
                    let player = world.get::<Actor>(s.player).unwrap().position;
                    let occupied: Vec<_> =
                        living(world).iter().map(|(_, a, _)| a.position).collect();
                    let position = s
                        .navigation
                        .cells
                        .iter()
                        .map(|cell| Vec2::from(cell.position))
                        .filter(|pos| {
                            pos.distance(player) >= 26.
                                && occupied.iter().all(|other| pos.distance(*other) >= 20.)
                                && s.navigation.clear(actor.position, *pos)
                        })
                        .min_by(|a, b| {
                            a.distance_squared(actor.position)
                                .total_cmp(&b.distance_squared(actor.position))
                        });
                    let Some(position) = position else {
                        continue;
                    };
                    let kind = summon.choose(s.random());
                    let summoned = &d.monsters[kind];
                    let dungeon = &d.dungeons[s.dungeon.as_ref().unwrap()];
                    let mode = &d.dungeon_modes[if s.extreme { "extreme" } else { "normal" }];
                    let hp = (summoned.hp
                        * (1. + s.wave as f32 * dungeon.scaling.hp_per_wave)
                        * mode.hp)
                        .round();
                    let damage = (summoned.damage
                        * (1. + s.wave as f32 * dungeon.scaling.damage_per_wave))
                        .round();
                    world.spawn((
                        Actor::new(position, hp),
                        Enemy {
                            kind: kind.into(),
                            damage,
                            spawned: s.time,
                            attack_at: s.time + 0.6,
                            path_at: 0.,
                            dead_at: 0.,
                            ranged_target: None,
                            ranged_until: 0.,
                        },
                    ));
                    float(
                        world,
                        "Invocation".into(),
                        position,
                        FloatKind::Reward,
                        None,
                    );
                    s.message(format!("{} invoque un {}.", def.name, summoned.name));
                    spawned += 1;
                }
                if spawned > 0 {
                    if let Some(buff) = &summon.ally_attack_buff {
                        for (ally, a, _) in living(world) {
                            if ally == id {
                                continue;
                            }
                            let previous = world
                                .get::<AllyAttackBonus>(ally)
                                .map_or(0., |bonus| bonus.0);
                            let next = (previous + buff.amount).min(buff.maximum);
                            let added = (next - previous).max(0.);
                            if added > 0. {
                                world.get_mut::<Enemy>(ally).unwrap().damage += added;
                                world.entity_mut(ally).insert(AllyAttackBonus(next));
                                float(
                                    world,
                                    format!("+{added:.0} dégâts"),
                                    a.position,
                                    FloatKind::Reward,
                                    None,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
