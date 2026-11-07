use super::*;

pub fn burst_seconds(b: &Burst, age: f32) -> f32 {
    let elapsed = (age - b.interval).max(0.);
    (elapsed / b.interval).floor() * b.duration + b.duration.min(elapsed % b.interval)
}
pub(super) fn enemies(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    m: &Manifest,
    dt: f32,
) {
    let visible = s.time >= s.invisible_until;
    if visible {
        s.last_known_player = world.get::<Actor>(s.player).unwrap().position;
    }
    let player_pos = s.last_known_player;
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    for id in ids {
        let mut a = world.get::<Actor>(id).unwrap().clone();
        let previous_position = a.position;
        let mut enemy = world.get::<Enemy>(id).unwrap().clone();
        a.animation += dt;
        if a.hp <= 0. {
            if s.time - enemy.dead_at >= d.balance.enemy_death_animation {
                world.despawn(id);
            } else {
                *world.get_mut::<Actor>(id).unwrap() = a;
            }
            continue;
        }
        let def = &d.monsters[&enemy.kind];
        if let (Some((target, release)), Some(r)) =
            (enemy.ranged_target, def.ranged_attack.as_ref())
        {
            if s.time >= release {
                world.spawn(Shot {
                    spell: String::new(),
                    position: a.position,
                    start: a.position,
                    target,
                    amount: enemy.damage,
                    speed: r.speed,
                    radius: r.radius,
                    hit: default(),
                    age: 0.,
                    hostile: true,
                    asset: r.asset.clone(),
                });
                enemy.ranged_target = None;
            }
        }
        if s.time >= enemy.ranged_until {
            let length = a.position.distance(player_pos);
            if length > def.detect {
                a.action("idle");
            } else {
                a.direction = direction(player_pos - a.position);
                let reach = def
                    .ranged_attack
                    .as_ref()
                    .map(|r| r.range)
                    .unwrap_or(def.range);
                if !visible && length < 3. {
                    a.route.clear();
                    a.action("idle");
                } else if visible && length <= reach && s.navigation.clear(a.position, player_pos) {
                    a.route.clear();
                    if s.time >= enemy.attack_at {
                        enemy.attack_at = s.time + def.interval;
                        a.until = s.time + d.balance.enemy_attack_animation;
                        a.action("attack");
                        a.animation = 0.;
                        if let Some(r) = def.ranged_attack.as_ref().filter(|_| length > def.range) {
                            enemy.ranged_target = Some((player_pos, s.time + r.windup));
                            enemy.ranged_until = s.time + r.animation_duration;
                            a.until = enemy.ranged_until;
                        } else {
                            damage_player(world, s, d, def.flat_damage.unwrap_or(enemy.damage));
                        }
                    } else if s.time >= a.until {
                        a.action("idle");
                    }
                } else if s.time >= a.until {
                    if s.time >= enemy.path_at {
                        enemy.path_at = s.time + 0.65;
                        enemy.path_at += (id.index() % 13) as f32 * 0.001;
                        enemy_route(&mut a, &mut s.navigation, player_pos);
                    }
                    let speed = if let Some(b) = &def.speed_burst {
                        let age = s.time - enemy.spawned;
                        def.speed
                            * (1.
                                + (b.multiplier - 1.)
                                    * (burst_seconds(b, age) - burst_seconds(b, age - dt))
                                    / dt)
                    } else {
                        def.speed
                    };
                    let moving = move_actor(&mut a, &s.navigation, speed, dt);
                    a.action(if moving { "run" } else { "idle" });
                }
            }
        }
        *world.get_mut::<Actor>(id).unwrap() = a;
        *world.get_mut::<Enemy>(id).unwrap() = enemy;
        if world.get::<Actor>(id).is_some_and(|actor| actor.hp > 0.) {
            super::sram::trigger_traps(
                world,
                s,
                p,
                d,
                m,
                Some((
                    id,
                    previous_position,
                    world.get::<Actor>(id).unwrap().position,
                )),
            );
        }
        if s.phase == Phase::Death {
            break;
        }
    }
}
pub(super) fn enemy_route(a: &mut Actor, nav: &mut Navigation, target: Vec2) {
    a.route = nav.route(a.position, target);
}
