use super::*;

pub(super) fn move_actor(a: &mut Actor, nav: &Navigation, speed: f32, dt: f32) -> bool {
    while a
        .route
        .front()
        .is_some_and(|p| a.position.distance(*p) < 0.5)
    {
        a.route.pop_front();
    }
    let Some(&target) = a.route.front() else {
        return false;
    };
    let delta = target - a.position;
    let next = a.position + delta.normalize_or_zero() * (speed * dt).min(delta.length());
    if !nav.clear(a.position, next) {
        a.route.clear();
        return false;
    }
    a.position = next;
    a.direction = direction(delta);
    true
}
pub(super) fn landing(nav: &mut Navigation, start: Vec2, target: Vec2, range: f32) -> Option<Vec2> {
    let delta = target - start;
    let mut travel = delta.length().min(range);
    while travel >= 1. {
        let point = start + delta.normalize_or_zero() * travel;
        if nav.contains(point) && !nav.route(start, point).is_empty() {
            return Some(point);
        }
        travel -= 2.;
    }
    None
}
