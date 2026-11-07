use bouftourust::{
    data::GameData,
    profile::{Profile, SaveStore},
};
fn data() -> GameData {
    GameData::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")).unwrap()
}
fn give(p: &mut Profile, item: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            p.give(item);
            p.inventory.last().unwrap().uid.clone()
        })
        .collect()
}
#[test]
fn twenty_seven_originals_produce_one_four_star_item_with_original_stats_times_three() {
    let d = data();
    let mut p = Profile::default();
    give(&mut p, "8254", 27);
    for (level, count, factor) in [(0, 9, 1.5), (1, 3, 2.), (2, 1, 3.)] {
        let ids: Vec<_> = p
            .inventory
            .iter()
            .filter(|i| i.fusion_level == level)
            .map(|i| i.uid.clone())
            .collect();
        for uid in ids.into_iter().step_by(3) {
            assert!(p.fuse(&uid, level, &d));
        }
        assert_eq!(p.inventory.len(), count);
        assert!(p.inventory.iter().all(|i| i.fusion_level == level + 1));
        for i in &p.inventory {
            assert_eq!(i.stats(&d).hp, d.items["8254"].stats.hp * factor);
            assert_eq!(i.stats(&d).power, d.items["8254"].stats.power * factor);
        }
    }
    let item = p.inventory[0].clone();
    assert_eq!(item.name(&d), format!("{} ****", d.items["8254"].name));
    assert!(!p.fuse(&item.uid, 3, &d));
    assert!(p.equip(&item.uid, Some("ring1"), &d));
    assert_eq!(
        p.stats(&d).hp,
        d.balance.starting_hp + d.items["8254"].stats.hp * 3.
    );
    p.validate(&d).unwrap();
}
#[test]
fn fusion_rejects_mixed_grades_worn_ingredients_and_replayed_requests() {
    let d = data();
    let mut p = Profile::default();
    let ids = give(&mut p, "8254", 3);
    p.equip(&ids[2], Some("ring1"), &d);
    let before = serde_json::to_string(&p).unwrap();
    assert!(!p.fuse(&ids[0], 0, &d));
    assert!(!p.fuse(&ids[2], 0, &d));
    assert_eq!(serde_json::to_string(&p).unwrap(), before);
    let other = give(&mut p, "8253", 1);
    assert!(!p.fuse(&ids[0], 0, &d));
    p.inventory
        .iter_mut()
        .find(|i| i.uid == ids[1])
        .unwrap()
        .fusion_level = 1;
    give(&mut p, "8254", 1);
    assert!(!p.fuse(&ids[0], 0, &d));
    give(&mut p, "8254", 1);
    assert!(p.fuse(&ids[0], 0, &d));
    assert_eq!(p.equipment["ring1"], ids[2]);
    assert!(p.inventory.iter().any(|i| i.uid == other[0]));
    let upgraded = give(&mut p, "8254", 1);
    p.inventory
        .iter_mut()
        .find(|i| i.uid == upgraded[0])
        .unwrap()
        .fusion_level = 1;
    assert!(p.fusion_plan(&ids[0]).is_some());
    assert!(
        !p.fuse(&ids[0], 0, &d),
        "A queued old click must not trigger the next grade"
    );
    assert!(p.fuse(&ids[0], 1, &d));
    assert_eq!(
        p.inventory
            .iter()
            .find(|i| i.uid == ids[0])
            .unwrap()
            .fusion_level,
        2
    );
}
#[test]
fn upgraded_items_survive_bulk_sale_and_keep_set_bonuses_unmultiplied() {
    let d = data();
    let mut p = Profile::default();
    for item in d.items.values().filter(|i| i.set == "mousse") {
        p.give(&item.id);
        let uid = p.inventory.last().unwrap().uid.clone();
        p.equip(&uid, None, &d);
    }
    let before = p.stats(&d);
    let counts = p.set_counts(&d);
    let added: f32 = p
        .inventory
        .iter()
        .map(|i| d.items[&i.item].stats.hp * 0.5)
        .sum();
    for item in &mut p.inventory {
        item.fusion_level = 1;
    }
    assert_eq!(p.stats(&d).hp, before.hp + added);
    assert_eq!(p.set_counts(&d), counts);
    let upgraded = give(&mut p, "8254", 2);
    for i in &mut p.inventory {
        if upgraded.contains(&i.uid) {
            i.fusion_level = 2;
        }
    }
    give(&mut p, "8254", 3);
    p.sell(&d);
    assert!(
        upgraded
            .iter()
            .all(|uid| p.inventory.iter().any(|i| &i.uid == uid))
    );
    assert!(
        p.equipment
            .values()
            .all(|uid| p.inventory.iter().any(|i| &i.uid == uid))
    );
}
#[test]
fn fusion_roundtrips_save_supports_old_instances_and_rejects_invalid_grade() {
    let d = data();
    let mut p: Profile =
        serde_json::from_str(r#"{"inventory":[{"uid":"old","item":"2469"}]}"#).unwrap();
    assert_eq!(p.inventory[0].fusion_level, 0);
    p.validate(&d).unwrap();
    give(&mut p, "2469", 2);
    assert!(p.fuse("old", 0, &d));
    assert_eq!(p.inventory[0].stats(&d).cooldown_reduction, 15.);
    p.equip("old", Some("ring1"), &d);
    let path = std::env::temp_dir().join(format!("bouftourust-fusion-{}.json", std::process::id()));
    let mut store = SaveStore {
        path: path.clone(),
        dirty: true,
        error: None,
    };
    store.save(&p).unwrap();
    let mut restored = store.load(&d).unwrap();
    assert_eq!(restored.inventory[0].fusion_level, 1);
    assert_eq!(restored.stats(&d).cooldown_reduction, 15.);
    restored.inventory[0].fusion_level = 3;
    assert_eq!(restored.stats(&d).cooldown_reduction, 30.);
    restored.inventory[0].fusion_level = 4;
    assert!(restored.validate(&d).is_err());
    std::fs::remove_file(path).unwrap();
    let _ = std::fs::remove_file(store.path.with_extension("bak"));
}
