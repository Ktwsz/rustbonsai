pub mod utils;

use std::f64::consts::PI;
use rand_distr::{Normal, Uniform, Distribution};
use std::collections::VecDeque;
use rand;

const DT: f64 = 0.001;
const T: u32 = 1000;
const MAX_DIST: f64 = 2.5;
const MIN_DIST: f64 = 0.3;
const PHI_TOLERANCE: f64 = 5f64;
const PHI_NEIGH_TOLERANCE: f64 = 100f64;

pub struct BonsaiTree {
    nodes: Vec <utils::Triangle>,
    bounds: (u32, u32),
    animation_time: f64,
    current_frame: f64
}

pub enum AsciiChange {
    Start,
    Change((usize, usize), char),
    Stop,
}

impl BonsaiTree {
    pub fn new(bounds: (u32, u32), animation_time: f64) -> Self {
        BonsaiTree {
            nodes: Vec::new(),
            bounds,
            animation_time,
            current_frame: 0.0,
        }
    }

    fn push(&mut self, s: &utils::Point, t: &utils::Point, phi_offset: f64) {
        self.nodes.push(utils::Triangle::from_points(&s, &t, phi_offset));
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

        let (next_point, next_point_radius, next_point_bphi) = sample_point(&starting_point, 1.0, None, &self.nodes).unwrap();
        println!("first {:?}", next_point);

        self.push(&starting_point, &next_point, next_point_bphi);

        let mut que: VecDeque<(utils::Point, f64, u8)> = VecDeque::new();
        que.push_back((next_point, next_point_radius, 0));

        while !que.is_empty() {
            println!("generating...");
            let (point, dist, ctr) = que.pop_front().unwrap();

            let sample = sample_point(&point, dist, None, &self.nodes);

            let neigh: Option<utils::Point> = 
                if let Some((found_point1, next_point_radius, next_point_bphi)) = sample {
                    self.push(&point, &found_point1, next_point_bphi);

                    if MAX_DIST > MIN_DIST + dist + next_point_radius && ctr < 1 {
                        que.push_back((found_point1, dist + next_point_radius, ctr + 1));
                    }

                    Some(found_point1)
                } else { None };

            let sample2 = sample_point(&point, dist, neigh, &self.nodes);
            if let Some((found_point2, next_point_radius2, next_point_bphi2)) = sample2 {
                self.push(&point, &found_point2, next_point_bphi2);

                if MAX_DIST > MIN_DIST + dist + next_point_radius2 && ctr < 1 {
                    que.push_back((found_point2, dist + next_point_radius2, ctr + 1));
                }
            }
        }
    }

    pub fn animation_step(&mut self, dt: f64) -> Vec<AsciiChange> {
        if self.current_frame >= self.animation_time {
            return vec![AsciiChange::Stop];
        }

        self.current_frame += dt;
        let mut result: Vec<AsciiChange> = Vec::new();

        for t in &self.nodes {
            for p in t.bezier_interpolate_all(DT, T) {
                result.push(AsciiChange::Change((f64::floor(p.x) as usize, f64::floor(p.y) as usize), '*'));
            }
        }

        result
    }

    fn fill_buffer(&self, buffer: &mut Vec<Vec<char>>) {
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

fn sample_point(parent: &utils::Point, dist_left: f64, neigh: Option<utils::Point>, nodes: &Vec<utils::Triangle>) -> Option<(utils::Point, f64, f64)> {
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
        bezier_ix = 0;
        phi = uniform.sample(&mut rng) as f64 / 100000.0;

        while !tolerance_check(parent, phi, neigh) || !bounds_check(parent, phi, radius) {
            phi = uniform.sample(&mut rng) as f64 / 100000.0;
        }

        if iterations >= 100 {
            return None;
        }

        let min_y = 0.5;// if is_starter_point { 0.5 } else { 0.5 };

        let mut t = utils::Triangle::from_points(&parent, &parent.add_polar(phi, radius), bezier_phis[bezier_ix]);
        while iterations < 100 && !curve_check(&t, nodes, min_y) { 
            bezier_ix += 1;
            iterations += 1;

            if bezier_ix >= bezier_phis.len() {
                break;
            }

            t = utils::Triangle::from_points(&parent, &parent.add_polar(phi, radius), bezier_phis[bezier_ix]);
        }

        if bezier_ix >= bezier_phis.len() || iterations >= 100 {
            return None;
        }

        found = true;
        
    }    

    Some((parent.add_polar(phi, radius), radius, bezier_phis[bezier_ix]))
}

fn tolerance_check(parent: &utils::Point, phi: f64, neigh: Option<utils::Point>) -> bool {
    phi > utils::deg_to_rad(PHI_TOLERANCE) &&

    phi + utils::deg_to_rad(PHI_TOLERANCE) < 2.0 * PI &&

    f64::abs(parent.phi() - phi) > utils::deg_to_rad(PHI_TOLERANCE) &&

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

    if out_of_bounds {
        return false;
    }

    let intersects = nodes.iter()
        .filter(|v| t.intersects(v))
        .peekable()
        .peek()
        .is_some();

    !intersects
}
