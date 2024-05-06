pub mod utils;

use rand;
use rand::Rng;

use self::utils::Point;

const DT: f64 = 0.001;
const T: u32 = 1000;
const ANIMATION_STEP: f64 = 0.1;

const Y_GROWTH: i32 = 8;
const MAX_X_GROWTH: i32 = 3;
const BRANCHES_TIERS: i32 = 4;
const BRANCH_COOLDOWN: i32 = 2;

pub struct BonsaiTree {
    nodes: Vec <Point>,
    bounds: (u32, u32),

    rng: rand::rngs::ThreadRng,

    animation_queue: Vec <(usize, f64)>,
}

pub enum AsciiChange {
    Start,
    Change((usize, usize), char),
    Stop,
}

impl BonsaiTree {
    pub fn new(bounds: (u32, u32)) -> Self {
        BonsaiTree {
            nodes: Vec::new(),
            bounds,

            rng: rand::thread_rng(),

            animation_queue: vec![(0, 0.0)],
        }
    }

    fn push(&mut self, p: &Point) {
        self.nodes.push(*p);
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

        self.generate_tree(Point::from_floats(0.0, 0.0), Y_GROWTH, BRANCHES_TIERS, xdir);
    }

    fn generate_tree(&mut self, pos: Point, growth: i32, tier: i32, xdir: i32) {
        if tier == 0 {
            return;
        }

        if growth == 0 {
            self.generate_tree(pos, 1 << (tier - 1), tier - 1, xdir);

            return;
        }

        let mut next_pos = pos;

        for _ in 0..MAX_X_GROWTH {
            next_pos = next_pos + Point::from_floats((xdir * (self.rng.gen::<i32>() % 2)) as f64, 0.0);

            self.push(&next_pos);
        }

        next_pos = next_pos + Point::from_floats(0.0, 1.0);
        self.push(&next_pos);

        if growth % BRANCH_COOLDOWN == 0 && self.rng.gen::<i32>() % tier == 0 {
            self.generate_branch(next_pos, tier, xdir);
        }

        self.generate_tree(next_pos, growth - 1, tier, xdir);
    }

    fn generate_branch(&mut self, pos: Point, tier: i32, xdir: i32) {
        let next_dir = get_new_direction(xdir, &mut self.rng);

        if xdir > 0 {
            self.generate_tree(pos, 1 << (tier - 1), tier - 1, next_dir);
        }
    }

    pub fn animation_step(&mut self) -> Vec<AsciiChange> {
        if self.animation_queue.is_empty() {
            return vec![AsciiChange::Stop];
        }
        let mut result: Vec<AsciiChange> = Vec::new();

        let mut next_frame_queue: Vec <(usize, f64)> = Vec::new();

        self.animation_queue = next_frame_queue;

        result
    }

    pub fn fill_buffer(&self, buffer: &mut Vec<Vec<char>>) {
        for p in &self.nodes {
            buffer[f64::floor(p.x) as usize][f64::floor(p.y) as usize] = '*';
        }
    }
}

fn get_new_direction(dir: i32, rng: &mut rand::rngs::ThreadRng) -> i32 {
    let r = rng.gen::<i32>() & 2;

    if dir == 0 {
        if r == 0 { -1 } else { 1 }
    } else {
        dir * -1 * r
    }
}
