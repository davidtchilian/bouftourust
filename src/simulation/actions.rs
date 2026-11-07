use super::*;

pub(super) fn remove_transients(world: &mut World) {
    let ids: Vec<Entity> = world
        .query_filtered::<Entity, Or<(
            With<Enemy>,
            With<Shot>,
            With<SwordWave>,
            With<Effect>,
            With<Floating>,
        )>>()
        .iter(world)
        .collect();
    for id in ids {
        world.despawn(id);
    }
}
pub(super) fn change_map(
    world: &mut World,
    s: &mut Session,
    p: &Profile,
    d: &GameData,
    m: &Manifest,
    id: &str,
) {
    remove_transients(world);
    let map = m.map(id);
    s.map_id = id.into();
    s.revision += 1;
    s.destination = None;
    s.navigation = Navigation::new(&map.walkable);
    *world.get_mut::<Actor>(s.player).unwrap() = Actor::new(map.spawn.into(), p.stats(d).hp);
    s.buff_until = 0.;
    s.cast_until = 0.;
    s.cooldowns = [0.; 4];
    s.chastisement_until = 0.;
    s.combat_power = 0.;
    s.wrath = 1.;
    s.truce_until = 0.;
    s.glyphs.clear();
    s.traps.clear();
    s.invisible_until = 0.;
    s.last_known_player = map.spawn.into();
}
pub(super) fn process_actions(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    m: &Manifest,
) {
    let actions = std::mem::take(&mut world.resource_mut::<Actions>().0);
    for action in actions {
        match action {
            Action::Move(target) => {
                if !s.paused && !matches!(s.phase, Phase::Death | Phase::Victory) {
                    if s.destination
                        .is_some_and(|p| p.distance_squared(target) < 9.)
                        && !world.get::<Actor>(s.player).unwrap().route.is_empty()
                    {
                        continue;
                    }
                    s.destination = Some(target);
                    let start = world.get::<Actor>(s.player).unwrap().position;
                    let route = s.navigation.route(start, target);
                    world.get_mut::<Actor>(s.player).unwrap().route = route;
                }
            }
            Action::Stop => {
                s.destination = None;
                let mut actor = world.get_mut::<Actor>(s.player).unwrap();
                actor.route.clear();
                if actor.action == "run" {
                    actor.action("idle");
                }
            }
            Action::Cast(index, target) => cast(world, s, p, d, m, index, target),
            Action::Hub => {
                change_map(world, s, p, d, m, "hub");
                s.phase = Phase::Hub;
                s.dungeon = None;
                s.extreme = false;
                s.paused = false;
                world.resource_mut::<SaveStore>().dirty = true;
            }
            Action::Enter(id, extreme) => {
                if s.phase != Phase::Combat && d.dungeons.contains_key(&id) {
                    change_map(world, s, p, d, m, &d.dungeons[&id].map);
                    s.phase = Phase::Combat;
                    s.dungeon = Some(id);
                    s.extreme = extreme;
                    s.wave = -1;
                    s.waiting = true;
                    s.delay = d.balance.wave_delay;
                    s.paused = false;
                }
            }
            Action::Class(id) => {
                if s.phase == Phase::Hub && d.classes.contains_key(&id) {
                    p.class_id = id;
                    change_map(world, s, p, d, m, "hub");
                    s.message(format!(
                        "Classe : {}. Progression conservée.",
                        d.classes[&p.class_id].name
                    ));
                    world.resource_mut::<SaveStore>().dirty = true;
                }
            }
            Action::Equip(uid, slot) => {
                if p.equip(&uid, slot.as_deref(), d) {
                    let mut a = world.get_mut::<Actor>(s.player).unwrap();
                    a.max_hp = p.stats(d).hp;
                    a.hp = a.hp.min(a.max_hp);
                    world.resource_mut::<SaveStore>().dirty = true;
                }
            }
            Action::Unequip(slot) => {
                p.equipment.remove(&slot);
                let mut a = world.get_mut::<Actor>(s.player).unwrap();
                a.max_hp = p.stats(d).hp;
                a.hp = a.hp.min(a.max_hp);
                world.resource_mut::<SaveStore>().dirty = true;
            }
            Action::Preferences {
                help_seen,
                reduced_motion,
            } => {
                p.help_seen = help_seen;
                p.reduced_motion = reduced_motion;
                world.resource_mut::<SaveStore>().dirty = true;
            }
            Action::Sell => {
                let (n, k) = p.sell(d);
                if n > 0 {
                    s.message(format!("{n} doublons vendus : +{k} kamas."));
                    world.resource_mut::<SaveStore>().dirty = true;
                }
            }
            Action::Fuse(uid, level) => {
                if p.fuse(&uid, level, d) {
                    let item = p.inventory.iter().find(|item| item.uid == uid).unwrap();
                    s.message(format!("Fusion réussie : {}", item.name(d)));
                    world.resource_mut::<SaveStore>().dirty = true;
                }
            }
        }
    }
}
