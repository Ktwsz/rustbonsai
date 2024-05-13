pub mod utils;

use std::cmp::Ordering;

use rand;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::layout::Rect;

use utils::Point;

const ANIMATION_STEP: i32 = 100;
const DT: f64 = 0.001;

const Y_GROWTH: i32 = 4;
const MAX_X_GROWTH: i32 = 1;
const BRANCHES_TIERS: i32 = 2;
const BRANCH_COOLDOWN: i32 = 1;

const POT_HEIGHT: f64 = 1.0 / 7.0;

enum AnimationItem {
    Start,
    Tree(usize, usize, f64),
    Leaf(usize, usize),
}

pub enum PointType {
    Tree(Point),
    Leaf(Point),
}

pub struct BonsaiTree {
    nodes: Vec <Point>,
    leaves_preprocess: Vec <Vec <(Point, i32)>>,
    leaves: Vec <Vec <Point>>,

    pot: [Point; 4],

    tree_bounds: (u16, u16),
    bounds: (u16, u16),

    rng: StdRng,

    neighbours: Vec <Vec <usize>>,
    animation_queue: Vec <AnimationItem>,
}

impl BonsaiTree {
    pub fn new(bounds: Rect, seed: Option <u64>) -> Self {
        let tree_bounds = (bounds.width - bounds.x, f64::floor((1.0 - POT_HEIGHT) *(bounds.height - bounds.y) as f64) as u16);
        let bounds = (bounds.width - bounds.x , bounds.height - bounds.y);

        let pot = get_pot_points(bounds);

        BonsaiTree {
            nodes: Vec::new(),

            leaves_preprocess: Vec::new(),
            leaves: Vec::new(),

            pot,

            bounds,
            tree_bounds,

            rng: if let Some(s) = seed { StdRng::seed_from_u64(s) } else { StdRng::from_entropy() },

            neighbours: Vec::new(),
            animation_queue: vec![AnimationItem::Start],
        }
    }

    fn push(&mut self, p: &Point, parent: usize) -> usize {
        self.nodes.push(*p);

        self.neighbours.push(Vec::new());
        self.leaves.push(Vec::new());
        self.leaves_preprocess.push(Vec::new());
        if self.nodes.len() > 1 {
            self.neighbours[parent].push(self.nodes.len() - 1);

            self.nodes.len() - 1
        } else {
            0
        }
    }

    pub fn normalize(&mut self) {
        let min_x = self.nodes.iter().map(|p| p.x).fold(f64::MAX, |a, b| a.min(b));
        let max_x = self.nodes.iter().map(|p| p.x).fold(f64::MIN, |a, b| a.max(b));
        let min_y = self.nodes.iter().map(|p| p.y).fold(f64::MAX, |a, b| a.min(b));
        let max_y = self.nodes.iter().map(|p| p.y).fold(f64::MIN, |a, b| a.max(b));

        let bound_x = f64::max(f64::abs(min_x), f64::abs(max_x));

        let min_p = utils::Point::from_floats(-bound_x, min_y);
        let max_p = utils::Point::from_floats(bound_x, max_y);

        let offset_y = self.bounds.1 - self.tree_bounds.1;

        self.nodes.iter_mut().for_each(|v| {
            v.normalize(&min_p, &max_p, self.tree_bounds);
            *v = *v + Point::from_floats(0.0, offset_y as f64);
        });
    }

    pub fn generate(&mut self) {
        let xdir = if self.rng.gen::<i32>() % 2 == 0 { -1 } else { 1 };

        self.generate_tree(Point::from_floats(0.0, 0.0), Y_GROWTH, BRANCHES_TIERS, xdir, 0);
    }

    fn generate_tree(&mut self, pos: Point, growth: i32, tier: i32, xdir: i32, mut parent: usize) {
        if tier == 0 {
            self.generate_leaves(Point::from_floats(0.0, 0.0), 3, false, parent);
            return;
        }

        if growth == 0 {
            self.generate_tree(pos, 1 << (tier - 1), tier - 1, xdir, parent);
            return;
        }

        let mut next_pos = pos;

        for _ in 0..MAX_X_GROWTH {
            let grow_y = 0.1 * self.rng.gen_range(1..10) as f64;
            next_pos = next_pos + Point::from_floats(xdir as f64, grow_y);

            parent = self.push(&next_pos, parent);
        }

        if growth % BRANCH_COOLDOWN == 0 && self.rng.gen::<i32>() % tier == 0 {
            self.generate_branch(next_pos, tier, xdir, parent);
        }

        self.generate_tree(next_pos, growth - 1, tier, xdir, parent);
    }

    fn generate_branch(&mut self, pos: Point, tier: i32, xdir: i32, parent: usize) {
        let r = if self.rng.gen::<i32>() % 2 == 0 { -1 } else { 1 };

        let next_dir = xdir * r;

        self.generate_tree(pos, 1 << (tier - 1), tier - 1, next_dir, parent);
    }

    fn generate_leaves(&mut self, pos: Point, depth: u8, dir: bool, parent: usize) {
        if depth == 0 {
            return;
        }

        let circle_radius = 2 + self.rng.gen::<i32>() % 4;

        if depth == 3 {
            self.leaves_preprocess[parent].push((pos, circle_radius));

            let dir_y = self.rng.gen::<f64>() % 4.0;
            self.generate_leaves(pos + Point::from_floats(circle_radius as f64, dir_y), depth - 1, true, parent);

            let dir_y = self.rng.gen::<f64>() % 4.0;
            self.generate_leaves(pos + Point::from_floats(-circle_radius as f64, dir_y), depth - 1, false, parent);

            self.process_leaves(parent);
        } else {
            let dir_y = self.rng.gen::<f64>() % 4.0;

            let dir_x = if dir { 1.0 } else { -1.0 };

            self.leaves_preprocess[parent].push((pos + Point::from_floats(dir_x * circle_radius as f64, 0.0), circle_radius));

            let next_pos = pos + Point::from_floats(dir_x * 2.0 * circle_radius as f64, dir_y);

            self.generate_leaves(next_pos, depth - 1, dir, parent);
        }
    }

    fn process_leaves(&mut self, parent: usize) {
        for &(center, radius) in self.leaves_preprocess[parent].iter() {
            for x in -radius..=radius {
                for y in -radius..radius {
                    if center.y + y as f64 >= 0.0 && x*x + y*y <= radius*radius {
                        self.leaves[parent].push(center + Point::from_floats(x as f64, y as f64));
                    }
                }
            }
        }

        self.leaves[parent].sort_by(|&a, &b|
            if a.norm2() < b.norm2() { Ordering::Less }
            else if a.norm2() == b.norm2() { Ordering::Equal }
            else { Ordering::Greater }
        )
    }

    pub fn animation_step(&mut self) -> Vec<PointType> {
        if self.animation_queue.is_empty() {
            return Vec::new();
        }

        let mut result: Vec<PointType> = Vec::new();

        let mut next_frame_queue: Vec <AnimationItem> = Vec::new();

        for item in &self.animation_queue {
            match item {
                AnimationItem::Start => self.neighbours[0].iter().for_each(|&v| next_frame_queue.push(AnimationItem::Tree(0, v, 0.0))),
                &AnimationItem::Tree(parent, ix, dt) => {
                    for step in 0..ANIMATION_STEP {
                        let t = dt + step as f64 * DT;

                        let p = utils::linear_interpolate(&self.nodes[parent as usize], &self.nodes[ix], t);

                        result.push(PointType::Tree(p));
                    }

                    let next_dt = dt + ANIMATION_STEP as f64 * DT;
                    if f64::abs(1.0 - next_dt) <= 0.001 {
                        self.neighbours[ix].iter().for_each(|&v| next_frame_queue.push(AnimationItem::Tree(ix, v, 0.0)));

                        if !self.leaves[ix].is_empty() {
                            next_frame_queue.push(AnimationItem::Leaf(ix, 0));
                        }
                    } else {
                        next_frame_queue.push(AnimationItem::Tree(parent, ix, next_dt));
                    }
                }

                &AnimationItem::Leaf(parent, ix) => {

                    let range_end = usize::min(ix + 10, self.leaves[parent].len());
                    result.extend(self.leaves[parent][ix..range_end].iter().map(|&p| PointType::Leaf(self.nodes[parent] + p)));

                    if ix + 10 < self.leaves[parent].len() {
                        next_frame_queue.push(AnimationItem::Leaf(parent, ix + 10))
                    }
                }
            }
        }

        self.animation_queue = next_frame_queue;

        result
    }

    pub fn get_tree(&self) -> Vec <(f64, f64)> {
        let mut result: Vec <(f64, f64)> = Vec::new();

        for (parent, n) in self.neighbours.iter().enumerate() {
            for &child in n {
                (0..1000).map(|dt| utils::linear_interpolate(&self.nodes[parent], &self.nodes[child], dt as f64 / 1000.0))
                    .for_each(|p| result.push((p.x, p.y)));
            }
        }

        result
    }

    pub fn get_leaves(&self) -> Vec <(f64, f64)> {
        self.leaves.iter()
            .enumerate()
            .map(|(ix, v)| v.iter().map(move |&p| self.nodes[ix] + p))
            .flatten()
            .map(|p| (p.x, p.y))
            .collect()
    }

    pub fn get_pot(&self) -> Vec <(f64, f64)> {
        let p1 = self.pot.iter().fold(Point::from_floats(self.bounds.0 as f64 / 2.0, 0.0), |a, &b| if a.x > b.x || a.y > b.y { a } else { b });
        let p2 = self.pot.iter().fold(Point::from_floats(self.bounds.0 as f64 / 2.0, 0.0), |a, &b| if a.x < b.x || a.y > b.y { a } else { b });

        let squares = [
            (p1.x, p1.y + 1.0),
            (p1.x - 1.0, p1.y + 1.0),
            (p1.x, p1.y + 2.0),
            (p1.x - 1.0, p1.y + 2.0),
            (p2.x, p2.y + 1.0),
            (p2.x + 1.5, p2.y + 1.0),
            (p2.x, p2.y + 2.0),
            (p2.x + 1.5, p2.y + 2.0),
        ];

        let mut result: Vec <(f64, f64)> = std::iter::zip(self.pot.iter(), self.pot.iter().cycle().skip(1))
            .map(|(p1, p2)| (0..1000).map(|dt| utils::linear_interpolate(p1, p2, dt as f64 / 1000.0)))
            .flatten()
            .map(|p| (p.x, p.y))
            .collect();

        result.extend(squares);

        result
    }
}

fn get_pot_points(bounds: (u16, u16)) -> [Point; 4] {
    let bounds_f = (bounds.0 as f64, bounds.1 as f64);
    let center_x = bounds_f.0 * 0.5;

    let up = 1.0 / 5.0 * bounds_f.0;
    let down = 1.0 / 6.0 * bounds_f.0;

    [
        Point::from_floats(center_x - down, 0.0),
        Point::from_floats(center_x + down, 0.0),
        Point::from_floats(center_x + up, POT_HEIGHT * bounds_f.1),
        Point::from_floats(center_x - up, POT_HEIGHT * bounds_f.1),
    ]
}

