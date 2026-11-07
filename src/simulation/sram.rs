use super::*;
pub(super) fn mortal_multiplier(roll: f32) -> f32 {
    if roll < 0.5 { 3. } else { 1. }
}

#[derive(Clone)]
pub struct Trap {
    pub position: Vec2,
    pub amount: f32,
}

impl Trap {
    /// The isometric diamond becomes a square in these two coordinates.
    /// Segment clipping also catches a fast enemy crossing the cell in one step.
    pub fn crossed(&self, from: Vec2, to: Vec2) -> bool {
        let local = |p: Vec2| {
            let p = p - self.position;
            Vec2::new(p.x / 26.5 + p.y / 13.5, p.x / 26.5 - p.y / 13.5)
        };
        let start = local(from);
        let delta = local(to) - start;
        let (mut enter, mut leave) = (0_f32, 1_f32);
        for (origin, speed) in [(start.x, delta.x), (start.y, delta.y)] {
            if speed.abs() < 0.000001 {
                if origin.abs() > 1. {
                    return false;
                }
            } else {
                let a = (-1. - origin) / speed;
                let b = (1. - origin) / speed;
                enter = enter.max(a.min(b));
                leave = leave.min(a.max(b));
                if enter > leave {
                    return false;
                }
            }
        }
        true
    }
}

pub(super) fn trigger_traps(
    world: &mut World,
    s: &mut Session,
    p: &mut Profile,
    d: &GameData,
    m: &Manifest,
    movement: Option<(Entity, Vec2, Vec2)>,
) {
    if s.phase != Phase::Combat || s.paused || s.loading || s.traps.is_empty() {
        return;
    }
    let actors = living(world);
    let mut triggered = Vec::new();
    s.traps.retain(|trap| {
        let crossed = movement.is_some_and(|(_, from, to)| trap.crossed(from, to))
            || actors
                .iter()
                .any(|(_, a, _)| trap.crossed(a.position, a.position));
        if crossed {
            triggered.push(trap.clone());
        }
        !crossed
    });
    let mut flashes = BTreeSet::new();
    for trap in triggered {
        if flashes.insert((trap.position.x.to_bits(), trap.position.y.to_bits())) {
            effect(
                world,
                m,
                &d.spells["mass_trap"].effect,
                trap.position,
                0.,
                false,
            );
        }
        for (id, actor, _) in living(world) {
            if actor.position.distance(trap.position) <= d.spells["mass_trap"].radius
                || movement.is_some_and(|(enemy, from, to)| enemy == id && trap.crossed(from, to))
            {
                hit(world, s, p, d, id, trap.amount);
            }
        }
    }
}
