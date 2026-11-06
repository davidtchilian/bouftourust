use bevy::prelude::*;
use bouftourust::{assets::Manifest, data::*, navigation::Navigation, profile::*, simulation::*};
use std::{path::PathBuf, time::Duration};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}
fn data() -> GameData {
    GameData::load(&root()).unwrap()
}
#[test]
fn reference_progression() {
    let d = data();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("reference_progression.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut p: Profile = serde_json::from_value(case["profile"].clone()).unwrap();
        let actual = p.stats(&d);
        let expected = &case["stats"];
        assert_eq!(
            actual.hp,
            expected["hp"].as_f64().unwrap() as f32,
            "{}",
            case["setId"]
        );
        assert_eq!(actual.power, expected["power"].as_f64().unwrap() as f32);
        assert_eq!(
            actual.cooldown_reduction,
            expected["cooldownReduction"].as_f64().unwrap() as f32
        );
        assert_eq!(
            actual.movement_speed,
            case["speed"].as_f64().unwrap() as f32
        );
        assert_eq!(
            p.sell(&d),
            (0, 0),
            "Une seconde vente ne doit rien rapporter"
        );
    }
}
#[test]
fn reference_damage_and_xp() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("reference_progression.json")).unwrap();
    for case in fixture["damages"].as_array().unwrap() {
        assert_eq!(
            damage(
                16.,
                case["c"].as_f64().unwrap() as f32,
                case["power"].as_f64().unwrap() as f32,
                case["buff"].as_f64().unwrap() as f32
            ),
            case["value"].as_f64().unwrap() as f32,
            "{case}"
        );
    }
    for case in fixture["xp"].as_array().unwrap() {
        let mut p = Profile::default();
        let gained = p.award_xp(case["amount"].as_u64().unwrap() as u32, &data());
        assert_eq!(p.level, case["level"].as_u64().unwrap() as u32);
        assert_eq!(p.xp, case["xp"].as_u64().unwrap() as u32);
        assert_eq!(gained, case["hp"].as_f64().unwrap() as f32);
    }
}
#[test]
fn rings_and_duplicates() {
    let d = data();
    let mut p = Profile::default();
    for _ in 0..4 {
        p.give("2419");
    }
    let uids: Vec<_> = p.inventory.iter().map(|i| i.uid.clone()).collect();
    assert!(p.equip(&uids[0], Some("ring1"), &d));
    assert!(p.equip(&uids[1], Some("ring2"), &d));
    assert_eq!(p.set_counts(&d)["bouftou"].len(), 1);
    assert_eq!(p.sell(&d), (2, 2));
    assert_eq!(p.inventory.len(), 2);
    assert_eq!(p.sell(&d), (0, 0));
    assert!(!p.equip(&uids[0], Some("hat"), &d));
}
#[test]
fn local_save_roundtrip_and_reject_corruption() {
    let d = data();
    let dir = std::env::temp_dir().join(format!("bouftourust-test-{}", std::process::id()));
    let path = dir.join("profile.json");
    let mut store = SaveStore {
        path: path.clone(),
        dirty: true,
        error: None,
    };
    let mut p = Profile::default();
    p.give("2411");
    let uid = p.inventory[0].uid.clone();
    p.equip(&uid, None, &d);
    store.save(&p).unwrap();
    assert_eq!(store.load(&d).unwrap().equipment, p.equipment);
    p.kamas = 99;
    store.save(&p).unwrap();
    assert!(path.with_extension("bak").is_file());
    std::fs::write(&path, b"invalid").unwrap();
    assert!(store.load(&d).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"invalid");
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn navigation_source_cells() {
    let m = Manifest::load(&root()).unwrap();
    m.validate(&data()).unwrap();
    for map in std::iter::once(&m.hub).chain(m.maps.values()) {
        let mut nav = Navigation::new(&map.walkable);
        let spawn: Vec2 = map.spawn.into();
        assert!(nav.contains(spawn));
        for c in &map.walkable {
            let goal: Vec2 = c.position.into();
            let route = nav.route(spawn, goal);
            let mut anchor = spawn;
            for point in route {
                assert!(
                    nav.clear(anchor, point),
                    "Route traverse obstacle cellule {}",
                    c.id
                );
                anchor = point;
            }
        }
        assert!(!nav.contains(Vec2::new(-1000., -1000.)));
    }
}
fn game() -> App {
    let d = data();
    let m = Manifest::load(&root()).unwrap();
    let store = SaveStore {
        path: std::env::temp_dir().join(format!("bouftourust-sim-{}.json", std::process::id())),
        dirty: false,
        error: None,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(d)
        .insert_resource(m)
        .insert_resource(Profile::default())
        .insert_resource(store)
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .add_plugins(GamePlugin);
    app.update();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs_f64(1. / 60.));
    app
}
fn tick(app: &mut App, n: u32) {
    for _ in 0..n {
        app.world_mut().run_schedule(FixedUpdate);
    }
}
#[test]
fn waves_modes_pause_cast_and_retry() {
    let mut app = game();
    app.world_mut()
        .resource_mut::<Actions>()
        .0
        .push(Action::Enter("incarnam".into(), true));
    tick(&mut app, 120);
    let world = app.world_mut();
    let count = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .count();
    assert_eq!(count, 30);
    let s = world.resource::<Session>();
    let player = s.player;
    let time = s.time;
    let position = world.get::<Actor>(player).unwrap().position;
    world.resource_mut::<Session>().paused = true;
    tick(&mut app, 60);
    assert_eq!(app.world().resource::<Session>().time, time);
    assert_eq!(app.world().get::<Actor>(player).unwrap().position, position);
    app.world_mut().resource_mut::<Session>().paused = false;
    app.world_mut()
        .resource_mut::<Actions>()
        .0
        .push(Action::Cast(0, position + Vec2::X * 40.));
    tick(&mut app, 1);
    let end = app.world().resource::<Session>().cast_until;
    assert!(end > time);
    let cooldown = app.world().resource::<Session>().cooldowns[0];
    app.world_mut()
        .resource_mut::<Actions>()
        .0
        .push(Action::Cast(0, position + Vec2::X * 40.));
    tick(&mut app, 1);
    assert_eq!(app.world().resource::<Session>().cooldowns[0], cooldown);
    app.world_mut()
        .resource_mut::<Actions>()
        .0
        .push(Action::Hub);
    tick(&mut app, 1);
    assert_eq!(app.world().resource::<Session>().phase, Phase::Hub);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .count(),
        0
    );
    assert_eq!(app.world().get::<Actor>(player).unwrap().hp, 180.);
}
#[test]
fn boss_speed_integrates_boundaries() {
    let b = Burst {
        interval: 5.,
        duration: 1.,
        multiplier: 2.,
    };
    assert_eq!(burst_seconds(&b, 4.9), 0.);
    assert!((burst_seconds(&b, 5.2) - 0.2).abs() < 0.0001);
    assert_eq!(burst_seconds(&b, 10.5), 1.5);
}
#[test]
fn every_dungeon_completes_with_equipment() {
    let mut app = game();
    {
        let d = app.world().resource::<GameData>().clone();
        let mut p = app.world_mut().resource_mut::<Profile>();
        p.give("devHat");
        let uid = p.inventory[0].uid.clone();
        p.equip(&uid, None, &d);
    }
    let ids: Vec<_> = app
        .world()
        .resource::<GameData>()
        .dungeons
        .keys()
        .cloned()
        .collect();
    for id in ids {
        for extreme in [false, true] {
            app.world_mut()
                .resource_mut::<Actions>()
                .0
                .push(Action::Enter(id.clone(), extreme));
            tick(&mut app, 1);
            for frame in 0..36000 {
                if frame % 5 == 0 {
                    let world = app.world_mut();
                    let player = world.resource::<Session>().player;
                    let pos = world.get::<Actor>(player).unwrap().position;
                    let target = world
                        .query::<(&Actor, &Enemy)>()
                        .iter(world)
                        .filter(|(a, _)| a.hp > 0.)
                        .min_by(|(a, _), (b, _)| {
                            a.position
                                .distance_squared(pos)
                                .total_cmp(&b.position.distance_squared(pos))
                        })
                        .map(|(a, _)| a.position);
                    if let Some(target) = target {
                        let clear = world.resource::<Session>().navigation.clear(pos, target);
                        let mut actions = world.resource_mut::<Actions>();
                        if !clear {
                            actions.0.push(Action::Move(target));
                        } else {
                            actions.0.extend([
                                Action::Cast(3, target),
                                Action::Cast(0, target),
                                Action::Cast(1, target),
                                Action::Cast(2, target),
                            ]);
                        }
                    }
                }
                tick(&mut app, 1);
                if matches!(
                    app.world().resource::<Session>().phase,
                    Phase::Victory | Phase::Death
                ) {
                    break;
                }
            }
            assert_eq!(
                app.world().resource::<Session>().phase,
                Phase::Victory,
                "{id}, extreme={extreme}"
            );
            assert!(app.world().resource::<Profile>().completed.contains(&id));
            if extreme {
                let world = app.world();
                let d = world.resource::<GameData>();
                let p = world.resource::<Profile>();
                let set = &d.dungeons[&id].reward_set;
                for item in d.items.values().filter(|i| &i.set == set) {
                    assert!(p.inventory.iter().any(|i| i.item == item.id));
                }
            }
        }
    }
}

#[test]
fn other_classes_keep_their_four_abilities() {
    let mut app = game();
    for class in ["sacrier", "iop", "feca"] {
        app.world_mut().resource_mut::<Actions>().0.extend([
            Action::Hub,
            Action::Class(class.into()),
            Action::Enter("incarnam".into(), false),
        ]);
        tick(&mut app, 120);
        assert_eq!(app.world().resource::<Profile>().class_id, class);
        for index in 0..4 {
            let world = app.world_mut();
            let player = world.resource::<Session>().player;
            let victim = world
                .query::<(&Actor, &Enemy)>()
                .iter(world)
                .find(|(a, _)| a.hp > 0.)
                .map(|(a, _)| a.position)
                .unwrap();
            {
                let mut a = world.get_mut::<Actor>(player).unwrap();
                a.position = victim;
                a.hp = a.max_hp / 2.;
                a.route.clear();
            }
            let target = if (class == "iop" && index == 1) || (class == "feca" && index == 2) {
                world
                    .resource::<Session>()
                    .navigation
                    .cells
                    .iter()
                    .map(|c| Vec2::from(c.position))
                    .find(|p| p.distance(victim) > 20. && p.distance(victim) < 80.)
                    .unwrap()
            } else {
                victim
            };
            world
                .resource_mut::<Actions>()
                .0
                .push(Action::Cast(index, target));
            tick(&mut app, 1);
            let s = app.world().resource::<Session>();
            assert!(s.cooldowns[index] > s.time, "Classe {class}, sort {index}");
            tick(&mut app, 30);
        }
    }
}

#[test]
fn queued_save_flushes_on_exit() {
    let dir = std::env::temp_dir().join(format!("bouftourust-writer-{}", std::process::id()));
    let path = dir.join("profile.json");
    let writer = SaveWorker::new(path.clone());
    let mut p = Profile::default();
    p.kamas = 42;
    assert!(writer.enqueue(&p).unwrap());
    drop(writer);
    let store = SaveStore {
        path,
        dirty: false,
        error: None,
    };
    assert_eq!(store.load(&data()).unwrap().kamas, 42);
    std::fs::remove_dir_all(dir).unwrap();
}
