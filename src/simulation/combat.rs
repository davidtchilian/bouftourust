use super::*;

pub(super) fn float(
    world: &mut World,
    text: String,
    pos: Vec2,
    kind: FloatKind,
    icon: Option<String>,
) {
    world.spawn(Floating {
        position: pos,
        text,
        age: 0.,
        kind,
        icon,
    });
}
pub(super) fn effect(
    world: &mut World,
    m: &Manifest,
    asset: &str,
    pos: Vec2,
    angle: f32,
    follow: bool,
) {
    if let Some(a) = m.assets.get(asset) {
        world.spawn(Effect {
            asset: asset.into(),
            position: pos,
            age: 0.,
            ttl: a.frames.iter().map(|f| f.duration_ms).sum::<f32>() / 1000.,
            angle,
            follow,
        });
    }
}
pub(super) fn weighted(table: &[Drop], chance: f32, s: &mut Session) -> Option<String> {
    if table.is_empty() || s.random() >= chance {
        return None;
    }
    let total: f32 = table.iter().map(|e| e.weight.max(0.)).sum();
    if total <= 0. {
        return None;
    }
    let mut ticket = s.random() * total;
    for e in table {
        ticket -= e.weight.max(0.);
        if ticket < 0. {
            return Some(e.item.clone());
        }
    }
    table.last().map(|e| e.item.clone())
}
pub(super) fn hit(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    id: Entity,
    amount: f32,
) {
    if s.time < s.truce_until {
        return;
    }
    if world.get::<Enemy>(id).is_some_and(|enemy| {
        d.monsters[&enemy.kind]
            .invulnerability
            .as_ref()
            .is_some_and(|shield| shield.active(s.time - enemy.spawned))
    }) {
        return;
    }
    let Some(mut a) = world.get_mut::<Actor>(id) else {
        return;
    };
    if a.hp <= 0. {
        return;
    }
    let pos = a.position;
    a.hp = (a.hp - amount).max(0.);
    let dead = a.hp <= 0.;
    a.action(if dead { "death" } else { "hit" });
    a.until = s.time + d.balance.enemy_hit_animation;
    if dead {
        a.route.clear();
    }
    drop(a);
    float(world, format!("{amount:.0}"), pos, FloatKind::Damage, None);
    if !dead {
        return;
    }
    let mut enemy = world.get_mut::<Enemy>(id).unwrap();
    enemy.dead_at = s.time;
    enemy.ranged_target = None;
    let def = &d.monsters[&enemy.kind];
    drop(enemy);
    s.kills += 1;
    let gained = p.award_xp(def.xp, d);
    world.get_mut::<Actor>(s.player).unwrap().hp += gained;
    p.kamas += def.kamas;
    float(
        world,
        format!("+{} XP · {} k", def.xp, def.kamas),
        pos,
        FloatKind::Reward,
        None,
    );
    if gained > 0. {
        s.message(format!("Niveau {} !", p.level));
    }
    let chance = s
        .dungeon
        .as_ref()
        .map(|id| {
            if def.boss {
                d.dungeons[id].loot.boss_chance
            } else {
                d.dungeons[id].loot.normal_chance
            }
        })
        .unwrap_or(0.05);
    let mut drops = Vec::new();
    if let Some(i) = weighted(&def.loot, chance, s) {
        drops.push(i);
    }
    if let Some(b) = &def.bonus_loot {
        if let Some(i) = weighted(&b.table, b.chance, s) {
            drops.push(i);
        }
    }
    for i in drops {
        p.give(&i);
        let name = &d.items[&i].name;
        s.message(format!("Butin : {name}"));
        float(
            world,
            name.clone(),
            pos,
            FloatKind::Loot,
            Some(d.items[&i].icon.clone()),
        );
    }
    world.resource_mut::<SaveStore>().dirty = true;
}
pub(super) fn damage_player(world: &mut World, s: &mut Session, d: &GameData, amount: f32) {
    if s.time < s.truce_until || s.phase != Phase::Combat {
        return;
    }
    let mut a = world.get_mut::<Actor>(s.player).unwrap();
    if s.time < s.chastisement_until {
        s.combat_power += a.hp.min(amount) * d.class_balance.chastisement_power_per_damage;
    }
    a.hp = (a.hp - amount).max(0.);
    let pos = a.position;
    let dead = a.hp <= 0.;
    if dead {
        a.route.clear();
        a.action("death");
        a.until = f32::INFINITY;
    } else if a.action != "cast" && a.action != "buff" {
        a.action("hit");
        a.until = s.time + 0.16;
    }
    drop(a);
    float(world, format!("{amount:.0}"), pos, FloatKind::Damage, None);
    if dead {
        s.phase = Phase::Death;
        s.glyphs.clear();
        s.traps.clear();
        s.invisible_until = 0.;
        s.combat_power = 0.;
        s.truce_until = 0.;
        let shots: Vec<_> = world
            .query_filtered::<Entity, Or<(With<Shot>, With<SwordWave>)>>()
            .iter(world)
            .collect();
        for e in shots {
            world.despawn(e);
        }
        s.message("Vous êtes tombé. Tous les gains sont conservés.");
        world.resource_mut::<SaveStore>().dirty = true;
    }
}
pub(super) fn class_damage(
    world: &World,
    s: &Session,
    p: &Profile,
    d: &GameData,
    coefficient: f32,
) -> f32 {
    let a = world.get::<Actor>(s.player).unwrap();
    let ratio = a.hp / p.stats(d).hp;
    let bonus = if p.class_id == "sacrier" {
        d.class_balance.low_hp_maximum_bonus
            * ((1. - ratio) / (1. - d.class_balance.low_hp_threshold)).clamp(0., 1.)
    } else {
        0.
    };
    damage(
        d.classes[&p.class_id].base_damage,
        coefficient,
        p.stats(d).power + s.combat_power,
        (1. + bonus)
            * if p.class_id == "iop" && s.time < s.buff_until {
                d.spells["iop_power"].buff_multiplier.unwrap_or(1.)
            } else {
                1.
            },
    )
}
pub(super) fn cast(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    m: &Manifest,
    index: usize,
    target: Vec2,
) {
    if index >= 4 || s.paused || matches!(s.phase, Phase::Death | Phase::Victory) {
        return;
    }
    let class = &d.classes[&p.class_id];
    let id = &class.abilities[index];
    let spell = &d.spells[id];
    if s.time < s.cooldowns[index]
        || (spell.kind != "buff" && s.time < s.cast_until)
        || (p.class_id != "cra" && s.phase != Phase::Combat)
    {
        return;
    }
    let start = world.get::<Actor>(s.player).unwrap().position;
    let enemies = living(world);
    let victim = enemies.iter().min_by(|(ea, a, _), (eb, b, _)| {
        a.position
            .distance_squared(target)
            .total_cmp(&b.position.distance_squared(target))
            .then_with(|| ea.cmp(eb))
    });
    let range = spell.range.unwrap_or(f32::INFINITY);
    if matches!(
        spell.kind.as_str(),
        "swap" | "direct" | "punishment" | "mortal"
    ) && victim.is_none_or(|(_, a, _)| {
        (spell.kind != "swap" && a.position.distance(target) > d.class_balance.target_radius)
            || a.position.distance(start) > range
    }) {
        return;
    }
    if spell.kind == "swap"
        && victim.is_some_and(|(_, a, _)| {
            !s.navigation.contains(a.position) || s.navigation.route(start, a.position).is_empty()
        })
    {
        return;
    }
    let dest = if spell.kind == "teleport" {
        landing(&mut s.navigation, start, target, range)
    } else {
        None
    };
    if spell.kind == "teleport" && dest.is_none() {
        return;
    }
    if matches!(spell.kind.as_str(), "glyph" | "wrath")
        && (!s.navigation.contains(target) || start.distance(target) > range)
    {
        return;
    }
    if spell.kind == "wrath" && !s.navigation.clear(start, target) {
        return;
    }
    if p.class_id == "sram"
        && matches!(spell.kind.as_str(), "direct" | "mortal")
        && victim.is_some_and(|(_, a, _)| !s.navigation.clear(start, a.position))
    {
        return;
    }
    if spell.kind == "trap" && !s.navigation.contains(target) {
        return;
    }
    s.cooldowns[index] = s.time + spell.cooldown * (1. - p.stats(d).cooldown_reduction / 100.);
    s.cast_until = s.time + d.balance.cast_lock;
    let mut a = world.get_mut::<Actor>(s.player).unwrap();
    a.until = s.time + d.balance.cast_animation;
    a.action(if spell.kind == "buff" { "buff" } else { "cast" });
    a.animation = 0.;
    let mut delta = target - start;
    if delta.length() < 1. {
        delta = Vec2::X;
    }
    a.direction = direction(delta);
    drop(a);
    if matches!(spell.kind.as_str(), "direct" | "mortal") {
        s.invisible_until = 0.;
    }
    if spell.kind == "invisibility" {
        s.last_known_player = start;
        s.invisible_until = s.time + spell.buff_duration.unwrap_or(4.);
        for mut enemy in world.query::<&mut Enemy>().iter_mut(world) {
            enemy.path_at = 0.;
        }
        effect(world, m, &spell.effect, start, 0., true);
        return;
    }
    if spell.kind == "trap" {
        let cell = s.navigation.cell_at(target).unwrap();
        s.traps.push(super::sram::Trap {
            position: s.navigation.cells[cell].position.into(),
            amount: class_damage(world, s, p, d, spell.coefficient),
        });
        return;
    }
    if spell.kind == "buff" {
        s.buff_until = s.time + spell.buff_duration.unwrap_or(d.balance.buff_duration);
        // Iop's persistent aura is rendered from the buff timer, including while moving.
        if p.class_id != "iop" {
            effect(world, m, &spell.effect, start, 0., true);
        }
        return;
    }
    if id == "sword" {
        world.spawn(SwordWave::new(
            start,
            start + delta.normalize() * range.min(delta.length()),
            class_damage(world, s, p, d, spell.coefficient),
        ));
        return;
    }
    if matches!(id.as_str(), "explosive" | "flaming" | "magic") {
        let travel = if spell.kind == "line" {
            range
        } else {
            range.min(delta.length())
        };
        world.spawn(Shot {
            spell: id.clone(),
            position: start,
            start,
            target: start + delta.normalize() * travel,
            amount: damage(
                class.base_damage,
                spell.coefficient,
                p.stats(d).power,
                if s.time < s.buff_until {
                    d.balance.buff_multiplier
                } else {
                    1.
                },
            ),
            speed: spell.speed,
            radius: spell.radius,
            hit: default(),
            age: 0.,
            hostile: false,
            asset: format!("spell.{id}.projectile"),
        });
        return;
    }
    let amount = class_damage(world, s, p, d, spell.coefficient);
    let mut hits = Vec::new();
    match spell.kind.as_str() {
        "drain" => {
            let mut drained = 0.;
            for (e, a, _) in &enemies {
                if a.position.distance(start) <= spell.radius
                    && s.navigation.clear(start, a.position)
                {
                    if s.time >= s.truce_until {
                        drained += a.hp.min(amount);
                    }
                    hits.push((*e, amount));
                }
            }
            let mut a = world.get_mut::<Actor>(s.player).unwrap();
            a.hp = (a.hp + (drained * d.class_balance.drain_ratio).floor()).min(p.stats(d).hp);
        }
        "chastisement" => s.chastisement_until = s.time + d.class_balance.chastisement_duration,
        "swap" => {
            if let Some((e, a, _)) = victim {
                let pos = a.position;
                let mut a = world.get_mut::<Actor>(*e).unwrap();
                a.position = start;
                a.route.clear();
                let mut enemy = world.get_mut::<Enemy>(*e).unwrap();
                enemy.path_at = 0.;
                enemy.ranged_target = None;
                let mut a = world.get_mut::<Actor>(s.player).unwrap();
                a.position = pos;
                a.route.clear();
                effect(world, m, &spell.effect, start, 0., false);
            }
        }
        "punishment" => {
            if let Some((e, _, _)) = victim {
                let ratio = world.get::<Actor>(s.player).unwrap().hp / p.stats(d).hp;
                hits.push((
                    *e,
                    class_damage(
                        world,
                        s,
                        p,
                        d,
                        spell.coefficient * (1. - (ratio - 0.5).abs() * 2.),
                    ),
                ));
            }
        }
        "wrath" => {
            let amount = class_damage(world, s, p, d, spell.coefficient * s.wrath);
            for (e, a, _) in &enemies {
                if a.position.distance(target) <= spell.radius
                    && s.navigation.clear(target, a.position)
                {
                    hits.push((*e, amount));
                }
            }
            s.wrath *= d.class_balance.wrath_growth;
        }
        "cone" => {
            for (e, a, _) in &enemies {
                let v = a.position - start;
                let len = v.length();
                if len <= range
                    && (len < 1.
                        || v.normalize().dot(delta.normalize()) >= d.class_balance.cone_cosine)
                    && s.navigation.clear(start, a.position)
                {
                    hits.push((*e, amount));
                }
            }
        }
        "teleport" => {
            if let Some(pos) = dest {
                let mut a = world.get_mut::<Actor>(s.player).unwrap();
                a.position = pos;
                a.route.clear();
            }
        }
        "push" => {
            for (e, a, _) in &enemies {
                if a.position.distance(start) <= spell.radius
                    && s.navigation.clear(start, a.position)
                {
                    hits.push((*e, amount));
                    let delta = (a.position - start)
                        .try_normalize()
                        .unwrap_or(delta.normalize());
                    let mut pos = a.position;
                    for _ in 0..(d.class_balance.push_distance / 2.) as u32 {
                        let next = pos + delta * 2.;
                        if !s.navigation.clear(pos, next) {
                            break;
                        }
                        pos = next;
                    }
                    let mut a = world.get_mut::<Actor>(*e).unwrap();
                    a.position = pos;
                    a.route.clear();
                    world.get_mut::<Enemy>(*e).unwrap().path_at = 0.;
                }
            }
        }
        "glyph" => s.glyphs.push(Glyph {
            position: target,
            next_tick: s.time + d.class_balance.glyph_interval,
        }),
        "direct" => {
            if let Some((e, _, _)) = victim {
                hits.push((*e, amount));
            }
        }
        "mortal" => {
            if let Some((e, _, _)) = victim {
                let multiplier = super::sram::mortal_multiplier(s.random());
                if multiplier > 1. {
                    float(
                        world,
                        "Critique ×3".into(),
                        victim.unwrap().1.position + Vec2::Y * 22.,
                        FloatKind::Damage,
                        None,
                    );
                }
                hits.push((*e, amount * multiplier));
            }
        }
        "truce" => s.truce_until = s.time + d.class_balance.truce_duration,
        _ => {}
    }
    for (e, n) in hits {
        hit(world, s, p, d, e, n);
    }
    if spell.kind != "glyph" && !spell.effect.is_empty() {
        let pos = if spell.kind == "wrath" {
            target
        } else if spell.kind == "cone" {
            start + delta.normalize() * range / 2.
        } else if matches!(spell.kind.as_str(), "direct" | "mortal") {
            victim.map(|(_, a, _)| a.position).unwrap_or(start)
        } else {
            world.get::<Actor>(s.player).unwrap().position
        };
        effect(world, m, &spell.effect, pos, 0., false);
    }
}
pub(super) fn update_shots(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    m: &Manifest,
    dt: f32,
) {
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<Shot>>()
        .iter(world)
        .collect();
    for e in ids {
        let Some(mut shot) = world.get::<Shot>(e).cloned() else {
            continue;
        };
        if shot.hostile && s.phase != Phase::Combat {
            world.despawn(e);
            continue;
        }
        let old = shot.position;
        let remaining = old.distance(shot.target);
        let travel = (shot.speed * dt).min(remaining);
        shot.position += (shot.target - old).normalize_or_zero() * travel;
        shot.age += dt;
        let blocked = !s.navigation.clear(old, shot.position);
        let mut arrived = remaining <= travel + 0.1 || blocked;
        if shot.hostile {
            if !blocked
                && segment_distance(
                    world.get::<Actor>(s.player).unwrap().position,
                    old,
                    shot.position,
                ) <= shot.radius
            {
                damage_player(world, s, d, shot.amount);
                arrived = true;
            }
        } else {
            let spell = &d.spells[&shot.spell];
            if spell.kind != "area" {
                for (id, a, _) in living(world) {
                    if !shot.hit.contains(&id)
                        && segment_distance(a.position, old, shot.position) <= spell.radius
                    {
                        shot.hit.insert(id);
                        hit(world, s, p, d, id, shot.amount);
                        effect(
                            world,
                            m,
                            &spell.effect,
                            a.position,
                            (shot.target - shot.start).to_angle(),
                            false,
                        );
                        if spell.kind == "single" {
                            arrived = true;
                            break;
                        }
                    }
                }
            }
            if arrived && spell.kind == "area" {
                for (id, a, _) in living(world) {
                    if a.position.distance(shot.position) <= spell.radius {
                        hit(world, s, p, d, id, shot.amount);
                    }
                }
                effect(world, m, &spell.effect, shot.position, 0., false);
            }
        }
        if arrived || s.phase == Phase::Death {
            world.despawn(e);
        } else if let Some(mut v) = world.get_mut::<Shot>(e) {
            *v = shot;
        }
    }
}
pub(super) fn glyphs(world: &mut World, s: &mut Session, p: &mut Profile, d: &GameData) {
    let mut ticks = Vec::new();
    for glyph in &mut s.glyphs {
        while glyph.next_tick <= s.time {
            if glyph.next_tick >= s.truce_until {
                ticks.push(glyph.position);
            }
            glyph.next_tick += d.class_balance.glyph_interval;
        }
    }
    for pos in ticks {
        let amount = class_damage(world, s, p, d, d.spells["glyph"].coefficient);
        for (e, a, _) in living(world) {
            if a.position.distance(pos) <= d.spells["glyph"].radius {
                hit(world, s, p, d, e, amount);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arena() -> (World, Session, Profile, GameData, Manifest) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let d = GameData::load(&root).unwrap();
        let m = Manifest::load(&root).unwrap();
        let mut p = Profile::default();
        p.class_id = "iop".into();
        let mut world = World::new();
        world.insert_resource(d.clone());
        world.insert_resource(m.clone());
        world.insert_resource(p.clone());
        super::super::setup(&mut world);
        let mut s = world.remove_resource::<Session>().unwrap();
        s.phase = Phase::Combat;
        s.loading = false;
        s.dungeon = Some("blopMulti".into());
        s.wave = 3;
        (world, s, p, d, m)
    }

    #[test]
    fn ancestral_summons_every_three_seconds_with_exact_rare_probability() {
        let (mut world, mut s, _, d, _) = arena();
        s.dungeon = Some("ancestral".into());
        // Keep this cadence test on ordinary summons; rare boundaries are checked below.
        s.random = 0xdeadbeef;
        let def = &d.monsters["ancestral"];
        assert!(def.speed <= 10.);
        let summon = def.summon.as_ref().unwrap();
        assert_eq!(summon.choose(0.), "ancestral");
        assert_eq!(summon.choose(0.004999), "ancestral");
        assert_ne!(summon.choose(0.005), "ancestral");
        assert_ne!(summon.choose(0.999999), "ancestral");
        let rare = (0..10000)
            .filter(|i| summon.choose((*i as f32 + 0.5) / 10000.) == "ancestral")
            .count();
        assert_eq!(rare, 50);
        let pos = world.get::<Actor>(s.player).unwrap().position;
        let boss = monster(&mut world, "ancestral", pos, def.hp);
        for (time, count) in [(2.999, 1), (3., 2), (3., 2), (6., 3)] {
            s.time = time;
            super::super::bosses::boss_abilities(&mut world, &mut s, &d);
            assert_eq!(living(&mut world).len(), count);
        }
        world.get_mut::<Actor>(boss).unwrap().hp = 0.;
        s.time = 9.;
        super::super::bosses::boss_abilities(&mut world, &mut s, &d);
        assert_eq!(living(&mut world).len(), 2);
    }

    #[test]
    fn meulou_summons_pairs_and_buffs_each_living_ally_once_up_to_300() {
        let (mut world, mut s, _, d, _) = arena();
        s.dungeon = Some("canides".into());
        let pos = world.get::<Actor>(s.player).unwrap().position;
        let boss = monster(&mut world, "meulou", pos, 11000.);
        let ally = monster(&mut world, "mulou", pos + Vec2::X * 30., 1700.);
        let dead = monster(&mut world, "mulou", pos + Vec2::Y * 30., 1700.);
        world.get_mut::<Actor>(dead).unwrap().hp = 0.;
        s.time = 4.999;
        super::super::bosses::boss_abilities(&mut world, &mut s, &d);
        assert_eq!(living(&mut world).len(), 2);
        for tick in 1..=8 {
            s.time = tick as f32 * 5.;
            super::super::bosses::boss_abilities(&mut world, &mut s, &d);
            assert_eq!(living(&mut world).len(), 2 + tick * 2);
            assert_eq!(
                world.get::<Enemy>(ally).unwrap().damage,
                1. + (tick as f32 * 50.).min(300.)
            );
            assert_eq!(world.get::<Enemy>(boss).unwrap().damage, 1.);
            assert_eq!(world.get::<Enemy>(dead).unwrap().damage, 1.);
            let newborns: Vec<_> = living(&mut world)
                .into_iter()
                .filter(|(_, _, e)| e.spawned == s.time)
                .collect();
            assert_eq!(newborns.len(), 2);
            for (id, _, enemy) in newborns {
                assert_eq!(enemy.kind, "mulou");
                assert_eq!(world.get::<AllyAttackBonus>(id).unwrap().0, 50.);
            }
            super::super::bosses::boss_abilities(&mut world, &mut s, &d);
            assert_eq!(living(&mut world).len(), 2 + tick * 2);
        }
        world.get_mut::<Actor>(boss).unwrap().hp = 0.;
        s.time = 45.;
        super::super::bosses::boss_abilities(&mut world, &mut s, &d);
        assert_eq!(living(&mut world).len(), 17);
        assert_eq!(world.get::<AllyAttackBonus>(ally).unwrap().0, 300.);
        let fresh = monster(&mut world, "mulou", pos, 1700.);
        assert!(world.get::<AllyAttackBonus>(fresh).is_none());
        assert_eq!(world.get::<Enemy>(fresh).unwrap().damage, 1.);
    }

    fn monster(world: &mut World, kind: &str, position: Vec2, hp: f32) -> Entity {
        world
            .spawn((
                Actor::new(position, hp),
                Enemy {
                    kind: kind.into(),
                    damage: 1.,
                    spawned: 0.,
                    attack_at: 0.,
                    path_at: 0.,
                    dead_at: 0.,
                    ranged_target: None,
                    ranged_until: 0.,
                },
            ))
            .id()
    }

    #[test]
    fn sram_traps_are_infinite_range_stackable_and_trigger_on_the_cell_only() {
        let (mut world, mut s, mut p, d, m) = arena();
        p.class_id = "sram".into();
        let start = world.get::<Actor>(s.player).unwrap().position;
        let target = s
            .navigation
            .cells
            .iter()
            .map(|c| Vec2::from(c.position))
            .find(|point| point.distance(start) > 250.)
            .unwrap();
        for time in [0., 2., 4.] {
            s.time = time;
            cast(&mut world, &mut s, &mut p, &d, &m, 1, target);
        }
        assert_eq!(s.traps.len(), 3);
        assert!(s.traps.iter().all(|trap| trap.position == target));
        let next_cell = monster(&mut world, "bwork", target + Vec2::X * 53., 10000.);
        let far = monster(&mut world, "bwork", target + Vec2::Y * 100., 10000.);
        super::super::sram::trigger_traps(&mut world, &mut s, &mut p, &d, &m, None);
        assert_eq!(
            s.traps.len(),
            3,
            "Être dans la zone d'explosion ne déclenche pas la case"
        );
        let trigger = monster(&mut world, "bwork", target, 10000.);
        let total: f32 = s.traps.iter().map(|trap| trap.amount).sum();
        super::super::sram::trigger_traps(&mut world, &mut s, &mut p, &d, &m, None);
        assert!(s.traps.is_empty());
        for id in [trigger, next_cell] {
            assert!((world.get::<Actor>(id).unwrap().hp - (10000. - total)).abs() < 0.01);
        }
        assert_eq!(world.get::<Actor>(far).unwrap().hp, 10000.);
    }

    #[test]
    fn sram_cell_selection_and_fast_crossing_use_the_isometric_diamond() {
        let (mut world, mut s, mut p, d, m) = arena();
        p.class_id = "sram".into();
        let center = Vec2::from(s.navigation.cells[0].position);
        let aim = center + Vec2::new(20., 2.);
        cast(&mut world, &mut s, &mut p, &d, &m, 1, aim);
        assert_eq!(s.traps[0].position, center);
        let amount = s.traps[0].amount;
        assert!(!s.traps[0].crossed(center + Vec2::Y * 14., center + Vec2::Y * 14.));
        let from = center - Vec2::X * 120.;
        let to = center + Vec2::X * 120.;
        let runner = monster(&mut world, "bwork", to, 10000.);
        super::super::sram::trigger_traps(
            &mut world,
            &mut s,
            &mut p,
            &d,
            &m,
            Some((runner, from, to)),
        );
        assert!(s.traps.is_empty());
        assert!((world.get::<Actor>(runner).unwrap().hp - (10000. - amount)).abs() < 0.01);
    }

    #[test]
    fn sram_invisibility_freezes_enemy_target_and_direct_attacks_reveal() {
        let (mut world, mut s, mut p, d, m) = arena();
        p.class_id = "sram".into();
        let start = world.get::<Actor>(s.player).unwrap().position;
        let enemy = monster(&mut world, "bwork", start, 10000.);
        cast(&mut world, &mut s, &mut p, &d, &m, 2, start);
        assert_eq!(s.invisible_until, 4.);
        let hp = world.get::<Actor>(s.player).unwrap().hp;
        let new_pos = start + Vec2::X * 10.;
        world.get_mut::<Actor>(s.player).unwrap().position = new_pos;
        s.time = 0.5;
        enemies(&mut world, &mut s, &mut p, &d, &m, 1. / 60.);
        assert_eq!(s.last_known_player, start);
        assert_eq!(world.get::<Actor>(s.player).unwrap().hp, hp);
        cast(&mut world, &mut s, &mut p, &d, &m, 1, new_pos);
        assert_eq!(s.invisible_until, 4., "Poser un piège ne révèle pas");
        s.time = 1.;
        cast(&mut world, &mut s, &mut p, &d, &m, 0, start);
        assert_eq!(s.invisible_until, 0.);
        enemies(&mut world, &mut s, &mut p, &d, &m, 1. / 60.);
        assert_eq!(s.last_known_player, new_pos);
        assert!(world.get::<Actor>(s.player).unwrap().hp < hp);
        assert!(world.get::<Actor>(enemy).unwrap().hp < 10000.);
        s.time = 15.;
        cast(&mut world, &mut s, &mut p, &d, &m, 2, new_pos);
        assert_eq!(s.invisible_until, 19.);
        s.time = 19.;
        world.get_mut::<Actor>(s.player).unwrap().position = start;
        enemies(&mut world, &mut s, &mut p, &d, &m, 1. / 60.);
        assert_eq!(
            s.last_known_player, start,
            "Expiration : le joueur est à nouveau suivi"
        );
    }

    #[test]
    fn sram_invalid_trap_does_not_spend_cooldown_and_map_change_cleans_statuses() {
        let (mut world, mut s, mut p, d, m) = arena();
        p.class_id = "sram".into();
        cast(&mut world, &mut s, &mut p, &d, &m, 1, Vec2::splat(-10000.));
        assert_eq!(s.cooldowns[1], 0.);
        assert!(s.traps.is_empty());
        let position = world.get::<Actor>(s.player).unwrap().position;
        cast(&mut world, &mut s, &mut p, &d, &m, 1, position);
        assert_eq!(s.traps.len(), 1);
        s.invisible_until = 10.;
        super::super::actions::change_map(&mut world, &mut s, &p, &d, &m, "hub");
        assert!(s.traps.is_empty());
        assert_eq!(s.invisible_until, 0.);
    }

    #[test]
    fn gelano_can_drop_even_when_the_normal_loot_table_is_empty() {
        let (mut world, mut s, mut p, mut d, _) = arena();
        world.insert_resource(SaveStore {
            path: std::path::PathBuf::from("unused-test-save.json"),
            dirty: false,
            error: None,
        });
        d.monsters.get_mut("blopCoco").unwrap().loot.clear();
        s.random = 42;
        let enemy = monster(&mut world, "blopCoco", Vec2::ZERO, 1.);
        hit(&mut world, &mut s, &mut p, &d, enemy, 2.);
        assert!(p.inventory.iter().any(|item| item.item == "2469"));
    }

    #[test]
    fn sram_mortal_attack_has_half_triple_hits_and_strict_short_range() {
        use super::super::sram::mortal_multiplier;
        assert_eq!(mortal_multiplier(0.499999), 3.);
        assert_eq!(mortal_multiplier(0.5), 1.);
        assert_eq!(
            (0..10000)
                .filter(|i| mortal_multiplier((*i as f32 + 0.5) / 10000.) == 3.)
                .count(),
            5000
        );
        let (mut world, mut s, mut p, d, m) = arena();
        p.class_id = "sram".into();
        let start = world.get::<Actor>(s.player).unwrap().position;
        let enemy = monster(&mut world, "bwork", start + Vec2::X * 100., 10000.);
        cast(
            &mut world,
            &mut s,
            &mut p,
            &d,
            &m,
            3,
            start + Vec2::X * 100.,
        );
        assert_eq!(s.cooldowns[3], 0.);
        world.get_mut::<Actor>(enemy).unwrap().position = start;
        s.random = 42; // First roll is below 0.5.
        let base = class_damage(&world, &s, &p, &d, d.spells["mortal_attack"].coefficient);
        cast(&mut world, &mut s, &mut p, &d, &m, 3, start);
        assert_eq!(world.get::<Actor>(enemy).unwrap().hp, 10000. - base * 3.);
        s.time = 5.;
        s.random = 0xdeadbeef;
        s.random(); // Next roll is above 0.5.
        cast(&mut world, &mut s, &mut p, &d, &m, 3, start);
        assert_eq!(world.get::<Actor>(enemy).unwrap().hp, 10000. - base * 4.);
    }

    #[test]
    fn iop_power_doubles_sword_and_wrath_for_exactly_five_seconds() {
        let (mut world, mut s, mut p, d, m) = arena();
        let position = world.get::<Actor>(s.player).unwrap().position;
        let base = class_damage(&world, &s, &p, &d, 9.);
        cast(&mut world, &mut s, &mut p, &d, &m, 2, position);
        assert_eq!(s.buff_until, 5.);
        assert_eq!(class_damage(&world, &s, &p, &d, 9.), base * 2.);
        s.time = 0.5;
        cast(
            &mut world,
            &mut s,
            &mut p,
            &d,
            &m,
            0,
            position + Vec2::X * 50.,
        );
        assert_eq!(
            world.query::<&SwordWave>().single(&world).unwrap().amount,
            damage(
                d.classes["iop"].base_damage,
                d.spells["sword"].coefficient,
                p.stats(&d).power,
                2.
            )
        );
        s.time = 4.999;
        assert_eq!(class_damage(&world, &s, &p, &d, 9.), base * 2.);
        s.time = 5.;
        assert_eq!(class_damage(&world, &s, &p, &d, 9.), base);
        assert_eq!(d.spells["power"].name, "Tir puissant");
    }

    #[test]
    fn wrath_hits_around_cursor_and_rejects_out_of_range_without_cooldown() {
        let (mut world, mut s, mut p, d, m) = arena();
        let target = s
            .navigation
            .cells
            .iter()
            .map(|c| Vec2::from(c.position))
            .find(|point| {
                s.navigation.contains(*point + Vec2::X * 50.)
                    && s.navigation
                        .clear(*point - Vec2::X * 50., *point + Vec2::X * 50.)
            })
            .unwrap();
        let start = target - Vec2::X * 50.;
        world.get_mut::<Actor>(s.player).unwrap().position = start;
        let center = monster(&mut world, "bwork", target, 10000.);
        let far_side = monster(&mut world, "bwork", target + Vec2::X * 50., 10000.);
        let outside = monster(&mut world, "bwork", target + Vec2::Y * 70., 10000.);
        cast(
            &mut world,
            &mut s,
            &mut p,
            &d,
            &m,
            3,
            start + Vec2::X * 100.,
        );
        assert_eq!(s.cooldowns[3], 0.);
        assert_eq!(s.wrath, 1.);
        cast(&mut world, &mut s, &mut p, &d, &m, 3, target);
        for id in [center, far_side] {
            assert_eq!(world.get::<Actor>(id).unwrap().hp, 10000. - 144.);
        }
        assert_eq!(world.get::<Actor>(outside).unwrap().hp, 10000.);
        assert_eq!(s.wrath, d.class_balance.wrath_growth);
        assert!(
            world
                .query::<&Effect>()
                .iter(&world)
                .any(|e| e.asset == "spell.wrath.cast" && e.position == target)
        );
    }

    #[test]
    fn bworkette_heals_living_allies_every_twelve_seconds_with_hp_cap() {
        let (mut world, mut s, _, d, _) = arena();
        let boss = monster(&mut world, "bworkette", Vec2::ZERO, 3400.);
        let ally = monster(&mut world, "bwork", Vec2::X, 1000.);
        let dead = monster(&mut world, "bwork", Vec2::Y, 1000.);
        world.get_mut::<Actor>(ally).unwrap().hp = 100.;
        world.get_mut::<Actor>(dead).unwrap().hp = 0.;
        for (time, expected) in [(11.99, 100.), (12., 600.), (12., 600.), (24., 1000.)] {
            s.time = time;
            super::super::bosses::boss_abilities(&mut world, &mut s, &d);
            assert_eq!(world.get::<Actor>(ally).unwrap().hp, expected);
            assert_eq!(world.get::<Actor>(dead).unwrap().hp, 0.);
        }
        world.get_mut::<Actor>(ally).unwrap().hp = 100.;
        world.get_mut::<Actor>(boss).unwrap().hp = 0.;
        s.time = 36.;
        super::super::bosses::boss_abilities(&mut world, &mut s, &d);
        assert_eq!(world.get::<Actor>(ally).unwrap().hp, 100.);
    }

    #[test]
    fn multicolored_blop_summons_one_royal_every_five_seconds_and_stops_when_dead() {
        let (mut world, mut s, _, d, _) = arena();
        let pos = world.get::<Actor>(s.player).unwrap().position;
        let boss = monster(&mut world, "royalBlopMulti", pos, 4200.);
        for (time, count) in [(4.99, 1), (5., 2), (5., 2), (10., 3)] {
            s.time = time;
            super::super::bosses::boss_abilities(&mut world, &mut s, &d);
            assert_eq!(living(&mut world).len(), count);
        }
        for (id, a, e) in living(&mut world) {
            if id != boss {
                assert!(
                    d.monsters["royalBlopMulti"]
                        .summon
                        .as_ref()
                        .unwrap()
                        .kinds
                        .contains(&e.kind)
                );
                assert!(s.navigation.contains(a.position));
            }
        }
        s.paused = true;
        s.time = 15.;
        super::super::bosses::boss_abilities(&mut world, &mut s, &d);
        assert_eq!(living(&mut world).len(), 3);
        s.paused = false;
        world.get_mut::<Actor>(boss).unwrap().hp = 0.;
        super::super::bosses::boss_abilities(&mut world, &mut s, &d);
        assert_eq!(living(&mut world).len(), 2);
    }

    #[test]
    fn golden_scara_blocks_damage_only_during_two_second_windows() {
        let (mut world, mut s, mut p, d, _) = arena();
        let boss = monster(&mut world, "goldScara", Vec2::ZERO, 10000.);
        for (time, expected) in [
            (4.99, 9900.),
            (5., 9900.),
            (6.99, 9900.),
            (7., 9800.),
            (10., 9800.),
            (12., 9700.),
        ] {
            s.time = time;
            hit(&mut world, &mut s, &mut p, &d, boss, 100.);
            assert_eq!(world.get::<Actor>(boss).unwrap().hp, expected);
        }
    }

    #[test]
    fn tofu_burst_and_gloutoblop_flat_contact_damage() {
        let (mut world, mut s, mut p, d, m) = arena();
        let burst = d.monsters["batofu"].speed_burst.as_ref().unwrap();
        assert_eq!(
            (burst.interval, burst.duration, burst.multiplier),
            (10., 1., 5.)
        );
        assert_eq!(burst_seconds(burst, 10.), 0.);
        assert_eq!(burst_seconds(burst, 11.), 1.);
        assert_eq!(burst_seconds(burst, 20.), 1.);
        let position = world.get::<Actor>(s.player).unwrap().position;
        let id = monster(&mut world, "gloutoblop", position, 1000.);
        world.get_mut::<Enemy>(id).unwrap().damage = 123.;
        world.get_mut::<Actor>(s.player).unwrap().hp = 2000000.;
        enemies(&mut world, &mut s, &mut p, &d, &m, 1. / 60.);
        assert_eq!(world.get::<Actor>(s.player).unwrap().hp, 1000000.);
    }

    #[test]
    fn iop_sword_wave_stays_static_and_hits_only_on_arrival() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let d = GameData::load(&root).unwrap();
        let m = Manifest::load(&root).unwrap();
        let mut p = Profile::default();
        p.class_id = "iop".into();
        let mut world = World::new();
        world.insert_resource(d.clone());
        world.insert_resource(m.clone());
        world.insert_resource(p.clone());
        super::super::setup(&mut world);
        let mut s = world.remove_resource::<Session>().unwrap();
        s.phase = Phase::Combat;
        let (start, target) = m
            .hub
            .walkable
            .iter()
            .find_map(|a| {
                m.hub.walkable.iter().find_map(|b| {
                    let start = Vec2::from(a.position);
                    let target = Vec2::from(b.position);
                    ((150. ..190.).contains(&start.distance(target))
                        && s.navigation.clear(start, target))
                    .then_some((start, target))
                })
            })
            .expect("Une trajectoire dégagée au-delà de l'ancienne portée");
        world.get_mut::<Actor>(s.player).unwrap().position = start;
        let direction = (target - start).normalize();
        let side = Vec2::new(-direction.y, direction.x);
        let mut spawn = |position| {
            world
                .spawn((
                    Actor::new(position, 1000.),
                    Enemy {
                        kind: d.monsters.keys().next().unwrap().clone(),
                        damage: 0.,
                        spawned: 0.,
                        attack_at: f32::INFINITY,
                        path_at: f32::INFINITY,
                        dead_at: 0.,
                        ranged_target: None,
                        ranged_until: 0.,
                    },
                ))
                .id()
        };
        let center = spawn(target);
        let nearby = spawn(target + side * 40.);
        let outside = spawn(target + side * 80.);
        let en_route = spawn(start + direction * 60.);
        cast(&mut world, &mut s, &mut p, &d, &m, 0, target);
        let wave = world.query::<&SwordWave>().single(&world).unwrap().clone();
        assert_eq!(world.query::<&Shot>().iter(&world).count(), 0);
        assert!(wave.target.distance(start) > 85.);
        assert!(wave.target.distance(start) <= 200.);
        update_sword_waves(&mut world, &mut s, &mut p, &d, &m, 0.1);
        for id in [center, nearby, outside, en_route] {
            assert_eq!(world.get::<Actor>(id).unwrap().hp, 1000.);
        }
        let first_effects: Vec<_> = world
            .query::<(Entity, &Effect)>()
            .iter(&world)
            .map(|(id, effect)| (id, effect.position))
            .collect();
        assert!(first_effects.len() >= 3);
        assert_eq!(first_effects[0].1, start);
        let player = s.player;
        world.get_mut::<Actor>(player).unwrap().position += side * 20.;
        for _ in 0..10 {
            update_sword_waves(&mut world, &mut s, &mut p, &d, &m, 0.1);
        }
        for id in [center, nearby] {
            assert_eq!(world.get::<Actor>(id).unwrap().hp, 1000. - wave.amount);
        }
        for id in [outside, en_route] {
            assert_eq!(world.get::<Actor>(id).unwrap().hp, 1000.);
        }
        assert_eq!(world.query::<&SwordWave>().iter(&world).count(), 0);
        for (id, position) in first_effects {
            assert_eq!(world.get::<Effect>(id).unwrap().position, position);
        }
        let effects: Vec<_> = world.query::<&Effect>().iter(&world).collect();
        assert!(effects.len() >= 10);
        assert!(
            effects
                .iter()
                .any(|effect| effect.position.distance(target) < 0.01)
        );
        assert!(effects.iter().all(|effect| !effect.follow));
    }
}
