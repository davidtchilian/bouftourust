use crate::assets::Cell;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
/// Graphe construit une fois par carte. Les monstres partagent un champ de distances.
#[derive(Clone)]
pub struct Navigation {
    pub cells: Vec<Cell>,
    buckets: HashMap<(i32, i32), Vec<usize>>,
    neighbors: Vec<Vec<usize>>,
    goal: usize,
    distances: Vec<f32>,
}
impl Navigation {
    pub fn new(cells: &[Cell]) -> Self {
        let mut n = Self {
            cells: cells.to_vec(),
            buckets: HashMap::new(),
            neighbors: vec![vec![]; cells.len()],
            goal: usize::MAX,
            distances: vec![f32::INFINITY; cells.len()],
        };
        for (i, c) in cells.iter().enumerate() {
            n.buckets
                .entry(Self::bucket(Vec2::from(c.position)))
                .or_default()
                .push(i);
        }
        for (i, c) in cells.iter().enumerate() {
            for (j, d) in cells.iter().enumerate() {
                let dx = (c.position[0] - d.position[0]).abs();
                let dy = (c.position[1] - d.position[1]).abs();
                if i != j
                    && dx <= 53.01
                    && dy <= 27.01
                    && dx / 26.5 + dy / 13.5 <= 2.01
                    && n.clear(c.position.into(), d.position.into())
                {
                    n.neighbors[i].push(j);
                }
            }
        }
        n
    }
    fn bucket(p: Vec2) -> (i32, i32) {
        ((p.x / 53.).floor() as i32, (p.y / 27.).floor() as i32)
    }
    pub fn nearest(&self, p: Vec2) -> usize {
        self.cells
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                Vec2::from(a.position)
                    .distance_squared(p)
                    .total_cmp(&Vec2::from(b.position).distance_squared(p))
            })
            .map(|(i, _)| i)
            .expect("Carte sans navigation")
    }
    pub fn depth(&self, p: Vec2) -> f32 {
        self.cells[self.nearest(p)].id as f32
    }
    pub fn contains(&self, p: Vec2) -> bool {
        let (x, y) = Self::bucket(p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cells) = self.buckets.get(&(x + dx, y + dy)) {
                    for &i in cells {
                        let c = self.cells[i].position;
                        if (c[0] - p.x).abs() / 26.5 + (c[1] - p.y).abs() / 13.5 <= 1.001 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    pub fn clear(&self, a: Vec2, b: Vec2) -> bool {
        let steps = (a.distance(b) / 3.).ceil() as u32;
        (0..=steps).all(|i| {
            self.contains(a.lerp(
                b,
                if steps == 0 {
                    0.
                } else {
                    i as f32 / steps as f32
                },
            ))
        })
    }
    pub fn route(&mut self, start: Vec2, target: Vec2) -> VecDeque<Vec2> {
        let goal = self.nearest(target);
        let end = if self.contains(target) {
            target
        } else {
            self.cells[goal].position.into()
        };
        if self.clear(start, end) {
            return VecDeque::from([end]);
        }
        if self.goal != goal {
            self.goal = goal;
            self.distances.fill(f32::INFINITY);
            self.distances[goal] = 0.;
            let mut open = vec![goal];
            while !open.is_empty() {
                let best = (0..open.len())
                    .min_by(|&a, &b| self.distances[open[a]].total_cmp(&self.distances[open[b]]))
                    .unwrap();
                let i = open.swap_remove(best);
                for &j in &self.neighbors[i] {
                    let next = self.distances[i]
                        + Vec2::from(self.cells[i].position)
                            .distance(self.cells[j].position.into());
                    if next < self.distances[j] {
                        self.distances[j] = next;
                        if !open.contains(&j) {
                            open.push(j);
                        }
                    }
                }
            }
        }
        let mut from = self.nearest(start);
        if !self.distances[from].is_finite() {
            return VecDeque::new();
        }
        let mut raw = vec![Vec2::from(self.cells[from].position)];
        while from != goal {
            let Some(&next) = self.neighbors[from]
                .iter()
                .filter(|&&j| self.distances[j] < self.distances[from])
                .min_by(|&&a, &&b| self.distances[a].total_cmp(&self.distances[b]))
            else {
                return VecDeque::new();
            };
            from = next;
            raw.push(self.cells[from].position.into());
        }
        raw.push(end);
        let mut path = VecDeque::new();
        let mut anchor = start;
        let mut i = 0;
        while i < raw.len() {
            let mut j = i;
            while j + 1 < raw.len() && self.clear(anchor, raw[j + 1]) {
                j += 1;
            }
            anchor = raw[j];
            path.push_back(anchor);
            i = j + 1;
        }
        path
    }
}
pub fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let d = b - a;
    let t = if d.length_squared() > 0. {
        ((p - a).dot(d) / d.length_squared()).clamp(0., 1.)
    } else {
        0.
    };
    p.distance(a + d * t)
}
