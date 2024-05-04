pub mod utils;

use std::f64::consts::PI;
use rand_distr::{Normal, Uniform, Distribution};
use std::collections::VecDeque;
use rand;
use rand::Rng;

const DT: f64 = 0.001;
const T: u32 = 1000;
const MAX_DIST: f64 = 2.5;
const MIN_DIST: f64 = 0.3;
const MAX_DEPTH: u8 = 1;
const PHI_TOLERANCE: f64 = 10.0;
const PHI_NEIGH_TOLERANCE: f64 = 100.0;
const ANIMATION_STEP: f64 = 0.1;

pub struct BonsaiTree {
    nodes: Vec <utils::Triangle>,
    bounds: (u32, u32),

    neighbours: Vec <Vec <usize>>,
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

            neighbours: vec![Vec::new()],
            animation_queue: vec![(0, 0.0)],
        }
    }

    fn push(&mut self, s: &utils::Point, t: &utils::Point, phi_offset: f64, parent: usize) {
        self.nodes.push(utils::Triangle::from_points(&s, &t, phi_offset));

        if self.nodes.len() > 1 {
            self.neighbours.push(Vec::new());
            let new_neigh = self.neighbours.len() - 1;
            self.neighbours[parent].push(new_neigh);
        }
    }

    pub fn normalize(&mut self) {
        let min_x = self.nodes.iter().map(|t| t[0].x.min(t[1].x.min(t[2].x))).fold(f64::MAX, |a, b| a.min(b));
        let max_x = self.nodes.iter().map(|t| t[0].x.max(t[1].x.max(t[2].x))).fold(f64::MIN, |a, b| a.max(b));
        let min_y = self.nodes.iter().map(|t| t[0].y.min(t[1].y.min(t[2].y))).fold(f64::MAX, |a, b| a.min(b));
        let max_y = self.nodes.iter().map(|t| t[0].y.max(t[1].y.max(t[2].y))).fold(f64::MIN, |a, b| a.max(b));

        let min_p = utils::Point::from_floats(min_x, min_y);
        let max_p = utils::Point::from_floats(max_x, max_y);
        self.nodes.iter_mut().for_each(|v| v.normalize(&min_p, &max_p, self.bounds));
    }

    pub fn generate(&mut self) {
        let starting_point = utils::Point::from_floats(1.0, 1.0);

        let (next_point, next_point_radius, next_point_bphi) = sample_point(&starting_point, 1.0, None, &self.nodes, true).unwrap();

        self.push(&starting_point, &next_point, next_point_bphi, 0);

        let mut que: VecDeque<(utils::Point, f64, u8, usize)> = VecDeque::new();
        que.push_back((next_point, next_point_radius, 0, 0));

        while !que.is_empty() {
            let (point, dist, depth, parent) = que.pop_front().unwrap();

            let sample = sample_point(&point, dist, None, &self.nodes, false);

            let neigh: Option<utils::Point> = 
                if let Some((found_point1, next_point_radius, next_point_bphi)) = sample {
                    self.push(&point, &found_point1, next_point_bphi, parent);

                    if MAX_DIST > MIN_DIST + dist + next_point_radius && depth < MAX_DEPTH {
                        que.push_back((found_point1, dist + next_point_radius, depth + 1, self.neighbours.len() - 1));
                    }

                    Some(found_point1)
                } else { None };

            let sample2 = sample_point(&point, dist, neigh, &self.nodes, false);
            if let Some((found_point2, next_point_radius2, next_point_bphi2)) = sample2 {
                self.push(&point, &found_point2, next_point_bphi2, parent);

                if MAX_DIST > MIN_DIST + dist + next_point_radius2 && depth < MAX_DEPTH {
                    que.push_back((found_point2, dist + next_point_radius2, depth + 1, self.neighbours.len() - 1));
                }
            }
        }
    }

    pub fn animation_step(&mut self) -> Vec<AsciiChange> {
        if self.animation_queue.is_empty() {
            return vec![AsciiChange::Stop];
        }
        let mut result: Vec<AsciiChange> = Vec::new();

        let mut next_frame_queue: Vec <(usize, f64)> = Vec::new();
        for (ix, dt) in &self.animation_queue {
            let end_dt = dt + ANIMATION_STEP;

            for p in self.nodes[*ix].bezier_interpolate_interval((*dt, end_dt), DT) {
                result.push(AsciiChange::Change((f64::floor(p.x) as usize, f64::floor(p.y) as usize), '&'))
            }

            if 1.0 - end_dt < 0.1 {
                self.neighbours[*ix].iter().for_each(|v| next_frame_queue.push((*v, 0.0)));
            } else {
                next_frame_queue.push((*ix, end_dt));
            }
        }

        self.animation_queue = next_frame_queue;

        result
    }

    pub fn fill_buffer(&self, buffer: &mut Vec<Vec<char>>) {
        for t in &self.nodes {
            for p in t.bezier_interpolate_all(DT, T) {
                buffer[f64::floor(p.x) as usize][f64::floor(p.y) as usize] = '*';
            }
        }
    }
}

fn fill_phi_values(phis: &mut Vec<f64>) {
    for val in (10..61).step_by(5).rev() {
        phis.push(utils::deg_to_rad(val as f64));
        phis.push(utils::deg_to_rad(-val as f64));
    }
}

fn sample_point(parent: &utils::Point, dist_left: f64, neigh: Option<utils::Point>, nodes: &Vec<utils::Triangle>, is_starter_point: bool) -> Option<(utils::Point, f64, f64)> {
    let normal = Normal::new(0.8 * (MAX_DIST - dist_left - MIN_DIST), 0.1 * MAX_DIST).unwrap();
    let uniform = Uniform::new(0, (2f64 * PI * 100000.0) as i32);

    let mut bezier_phis: Vec<f64> = Vec::new();
    fill_phi_values(&mut bezier_phis);

    let mut rng = rand::thread_rng();

    let radius = MIN_DIST * normal.sample(&mut rng);
    let mut phi = 0.0;
    let mut bezier_ix = 0;


    let mut found = false;
    let mut iterations = 0;
    while !found {
        bezier_ix = rng.gen_range(0..bezier_phis.len());
        phi = uniform.sample(&mut rng) as f64 / 100000.0;
        let mut t = utils::Triangle::from_points(&parent, &parent.add_polar(phi, radius), bezier_phis[bezier_ix]);

        let min_y = if is_starter_point { 0.5 } else { 1.5 };

        while iterations < 10000 && !tolerance_check(parent, phi, neigh) || !bounds_check(parent, phi, radius) && !curve_check(&t, nodes, min_y) {
            phi = uniform.sample(&mut rng) as f64 / 100000.0;
            bezier_ix = rng.gen_range(0..bezier_phis.len());

            t = utils::Triangle::from_points(&parent, &parent.add_polar(phi, radius), bezier_phis[bezier_ix]);

            iterations += 1;
        }

        if iterations >= 10000 {
            return None;
        }

        found = true;
    }    

    Some((parent.add_polar(phi, radius), radius, bezier_phis[bezier_ix]))
}

fn tolerance_check(parent: &utils::Point, phi: f64, neigh: Option<utils::Point>) -> bool {

    f64::abs(parent.phi() - phi) > utils::deg_to_rad(PHI_NEIGH_TOLERANCE) &&

    neigh.map_or(true, |n| f64::abs(n.phi() - phi) > utils::deg_to_rad(PHI_NEIGH_TOLERANCE))
}

fn bounds_check(parent: &utils::Point, phi: f64, radius: f64) -> bool {
    parent.add_polar(phi, radius).y > 1.0
}

fn curve_check(t: &utils::Triangle, nodes: &Vec<utils::Triangle>, min_y: f64) -> bool {
    let out_of_bounds = t.bezier_interpolate_all(DT, T).iter()
        .filter(|v| v.y < min_y)
        .peekable()
        .peek()
        .is_some();

    let intersects = nodes.iter()
        .filter(|v| t.intersects(v))
        .peekable()
        .peek()
        .is_some();

    !intersects && !out_of_bounds
}
