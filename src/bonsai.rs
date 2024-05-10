pub mod utils;

use rand;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::layout::Rect;

use utils::Point;

const ANIMATION_STEP: i32 = 500;

const Y_GROWTH: i32 = 8;
const MAX_X_GROWTH: i32 = 3;
const BRANCHES_TIERS: i32 = 4;
const BRANCH_COOLDOWN: i32 = 2;
const LEAF_HEIGHT: i32 = 3;
const LEAF_RADIUS: i32 = 6;

enum AnimationItem {
    Tree(isize, usize, f64),
    Leaf(usize, usize),
}

pub enum PointType {
    Tree(Point),
    Leaf(Point),
    Pot(Point),
}

pub struct BonsaiTree {
    nodes: Vec <Point>,
    leaves: Vec <Vec <Point>>,
    bounds: (u16, u16),

    rng: StdRng,

    neighbours: Vec <Vec <usize>>,
    animation_queue: Vec <AnimationItem>,
}

impl BonsaiTree {
    pub fn new(bounds: Rect, seed: Option <u64>) -> Self {
        BonsaiTree {
            nodes: Vec::new(),
            leaves: Vec::new(),
            bounds: (bounds.width - bounds.x, bounds.height - bounds.y),

            rng: if let Some(s) = seed { StdRng::seed_from_u64(s) } else { StdRng::from_entropy() },

            neighbours: Vec::new(),
            animation_queue: vec![AnimationItem::Tree(-1, 0, 0.0)],
        }
    }

    fn push(&mut self, p: &Point, parent: usize) -> usize {
        self.nodes.push(*p);

        self.neighbours.push(Vec::new());
        self.leaves.push(Vec::new());
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

        let min_p = utils::Point::from_floats(min_x, min_y);
        let max_p = utils::Point::from_floats(max_x, max_y);
        self.nodes.iter_mut().for_each(|v| v.normalize(&min_p, &max_p, self.bounds));
    }

    pub fn generate(&mut self) {
        let xdir = -1 + self.rng.gen_range(0..3);

        self.generate_tree(Point::from_floats(0.0, 0.0), Y_GROWTH, BRANCHES_TIERS, xdir, 0);
    }

    fn generate_tree(&mut self, pos: Point, growth: i32, tier: i32, xdir: i32, mut parent: usize) {
        if tier == 0 {
            self.generate_leaves(Point::from_floats(0.0, 0.0), LEAF_HEIGHT, LEAF_RADIUS, parent);
            return;
        }

        if growth == 0 {
            self.generate_tree(pos, 1 << (tier - 1), tier - 1, xdir, parent);
            return;
        }

        let mut next_pos = pos;

        for _ in 0..MAX_X_GROWTH {
            next_pos = next_pos + Point::from_floats((xdir * (self.rng.gen::<i32>() % 2)) as f64, 0.2);

            parent = self.push(&next_pos, parent);
        }

        parent = self.push(&next_pos, parent);

        if growth % BRANCH_COOLDOWN == 0 && self.rng.gen::<i32>() % tier == 0 {
            self.generate_branch(next_pos, tier, xdir, parent);
        }

        self.generate_tree(next_pos, growth - 1, tier, xdir, parent);
    }

    fn generate_branch(&mut self, pos: Point, tier: i32, xdir: i32, parent: usize) {
        let next_dir = get_new_direction(xdir, &mut self.rng);

        if xdir > 0 {
            self.generate_tree(pos, 1 << (tier - 1), tier - 1, next_dir, parent);
        }
    }

    fn generate_leaves(&mut self, pos: Point, height: i32, radius: i32, parent: usize) {
        if height <= 0 {
            return;
        }

        for x in -radius..=radius {
            if x != 0 && self.rng.gen::<i32>() % i32::abs(x) == 0 {
                self.leaves[parent].push(pos + Point::from_floats(x as f64, 1.0));
            }
        }

        self.generate_leaves(pos + Point::from_floats(0.0, 1.0), height - 1, radius - 1, parent);
    }

    pub fn animation_step(&mut self) -> Vec<PointType> {
        if self.animation_queue.is_empty() {
            return Vec::new();
        }

        let mut result: Vec<PointType> = Vec::new();

        let mut next_frame_queue: Vec <AnimationItem> = Vec::new();

        for item in &self.animation_queue {
            match item {
                &AnimationItem::Tree(parent, ix, dt) => {
                    if parent == -1 {
                        self.neighbours[ix].iter().for_each(|&v| next_frame_queue.push(AnimationItem::Tree(ix as isize, v, 0.0)));
                        continue;
                    }

                    for step in 0..ANIMATION_STEP {
                        let t = dt + step as f64 * 0.002;

                        let p = utils::linear_interpolate(&self.nodes[parent as usize], &self.nodes[ix], t);

                        result.push(PointType::Tree(p));
                    }

                    let next_dt = dt + ANIMATION_STEP as f64 * 0.002;
                    if 1.0 - next_dt <= 0.1 {
                        self.neighbours[ix].iter().for_each(|&v| next_frame_queue.push(AnimationItem::Tree(ix as isize, v, 0.0)));

                        if !self.leaves[ix].is_empty() {
                            next_frame_queue.push(AnimationItem::Leaf(ix, 0));
                        }
                    } else {
                        next_frame_queue.push(AnimationItem::Tree(parent, ix, next_dt));
                    }
                }

                &AnimationItem::Leaf(parent, ix) => {
                    result.push(PointType::Leaf(self.nodes[parent] + self.leaves[parent][ix]));
                    if ix + 1 < self.leaves[parent].len() {
                        next_frame_queue.push(AnimationItem::Leaf(parent, ix + 1))
                    }
                }
            }
        }

        self.animation_queue = next_frame_queue;

        result
    }

    pub fn get_leaves(&self) -> Vec <Point> {
        self.leaves.iter()
            .enumerate()
            .map(|(ix, v)| v.iter().map(move |value| self.nodes[ix] + *value))
            .flatten()
            .collect()
    }

    pub fn fill_buffer(&self, buffer: &mut Vec<Vec<char>>) {
        for p in &self.nodes {
            buffer[f64::floor(p.x) as usize][f64::floor(p.y) as usize] = '*';
        }
    }
}

fn get_new_direction(dir: i32, rng: &mut StdRng) -> i32 {
    let r = rng.gen::<i32>() & 2;

    if dir == 0 {
        if r == 0 { -1 } else { 1 }
    } else {
        dir * -1 * r
    }
}
