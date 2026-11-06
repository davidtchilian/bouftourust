use super::*;

pub(super) fn visuals_age(world: &mut World, s: &Session, dt: f32) {
    let pos = world.get::<Actor>(s.player).unwrap().position;
    let mut gone = Vec::new();
    for (e, mut effect) in world.query::<(Entity, &mut Effect)>().iter_mut(world) {
        effect.age += dt;
        if effect.follow {
            effect.position = pos;
        }
        if effect.age >= effect.ttl {
            gone.push(e);
        }
    }
    for (e, mut f) in world.query::<(Entity, &mut Floating)>().iter_mut(world) {
        f.age += dt;
        if f.age >= if f.kind == FloatKind::Loot { 1.8 } else { 1.2 } {
            gone.push(e);
        }
    }
    for e in gone {
        world.despawn(e);
    }
}
