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
        s.combat_power = 0.;
        s.truce_until = 0.;
        let shots: Vec<_> = world
            .query_filtered::<Entity, With<Shot>>()
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
        1. + bonus,
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
        || (id != "power" && s.time < s.cast_until)
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
    if matches!(spell.kind.as_str(), "swap" | "direct" | "punishment")
        && victim.is_none_or(|(_, a, _)| {
            (spell.kind != "swap" && a.position.distance(target) > d.class_balance.target_radius)
                || a.position.distance(start) > range
        })
    {
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
    if spell.kind == "glyph" && (!s.navigation.contains(target) || start.distance(target) > range) {
        return;
    }
    s.cooldowns[index] = s.time + spell.cooldown * (1. - p.stats(d).cooldown_reduction / 100.);
    s.cast_until = s.time + d.balance.cast_lock;
    let mut a = world.get_mut::<Actor>(s.player).unwrap();
    a.until = s.time + d.balance.cast_animation;
    a.action(if id == "power" { "buff" } else { "cast" });
    a.animation = 0.;
    let mut delta = target - start;
    if delta.length() < 1. {
        delta = Vec2::X;
    }
    a.direction = direction(delta);
    drop(a);
    if id == "power" {
        s.buff_until = s.time + d.balance.buff_duration;
        effect(world, m, &spell.effect, start, 0., true);
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
        "cone" | "wrath" => {
            for (e, a, _) in &enemies {
                let v = a.position - start;
                let len = v.length();
                if len <= range
                    && (len < 1.
                        || v.normalize().dot(delta.normalize()) >= d.class_balance.cone_cosine)
                    && s.navigation.clear(start, a.position)
                {
                    hits.push((
                        *e,
                        if spell.kind == "wrath" {
                            class_damage(world, s, p, d, spell.coefficient * s.wrath)
                        } else {
                            amount
                        },
                    ));
                }
            }
            if spell.kind == "wrath" {
                s.wrath *= d.class_balance.wrath_growth;
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
        "truce" => s.truce_until = s.time + d.class_balance.truce_duration,
        _ => {}
    }
    for (e, n) in hits {
        hit(world, s, p, d, e, n);
    }
    if spell.kind != "glyph" && !spell.effect.is_empty() {
        let pos = if matches!(spell.kind.as_str(), "cone" | "wrath") {
            start + delta.normalize() * range / 2.
        } else if spell.kind == "direct" {
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
