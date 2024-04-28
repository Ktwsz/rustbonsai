use std::ops::{Add, Sub, Mul, Div, Index, IndexMut};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64, 
    pub y: f64,
}

#[derive(Debug)]
pub struct Triangle {
    points: [Point; 3],
}

pub fn deg_to_rad(deg: f64) -> f64 {
    deg * 2f64 * PI / 360f64
}

impl Point {
    pub fn from_floats(x: f64, y: f64) -> Self {
        Point {x, y}
    }

    pub fn from_polar(phi: f64, radius: f64) -> Self {
        Point {
            x: f64::cos(phi),
            y: f64::sin(phi),
        } * radius
    }

    pub fn phi(&self) -> f64 {
        f64::atan(self.y / self.x)
    }

    pub fn radius(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y)
    }

    pub fn add_polar(&self, phi: f64, radius: f64) -> Self {
        *self + Point::from_polar(phi, radius)
    }
}

fn linear_interpolate(a: &Point, b: &Point, dt: f64) -> Point {
    *a + (*b - *a) * dt
}

impl Triangle {
    fn bezier_interpolate(&self, dt: f64) -> Point {
        let p1 = linear_interpolate(&self[0], &self[1], dt);
        let p2 = linear_interpolate(&self[1], &self[2], dt);

        linear_interpolate(&p1, &p2, dt)
    }

    pub fn bezier_interpolate_all(&self, dt: f64, t: u32) -> Vec<Point> {
        (0..t).map(|v| self.bezier_interpolate(dt * v as f64)).collect()
    }


    pub fn from_points(s: &Point, t: &Point, phi_offset: f64) -> Self {
        let v = *t - *s;

        let phi = v.phi() + phi_offset;
        let radius = v.radius();

        Triangle {
            points: [*s, Point::from_polar(phi, radius), *t]
        }
    }

    pub fn normalize(&mut self, min_p: &Point, max_p: &Point, bounds: (u32, u32)) {
        let normal_x = max_p.x - min_p.x;
        let normal_y = max_p.y - min_p.y;
        let normal_p = Point::from_floats(normal_x, normal_y);

        for i in 0..3 {
            self.points[i] = (self.points[i] - *min_p) / normal_p;
            self.points[i] = Point::from_floats(self.points[i].x * bounds.0 as f64, self.points[i].y * bounds.1 as f64);
        }
    }

    pub fn intersects(&self, t: &Triangle) -> bool {
        if line_intersect2(&self[0],&self[1],&t[0],&t[1]) { return true; }
        if line_intersect2(&self[0],&self[1],&t[0],&t[2]) { return true; }
        if line_intersect2(&self[0],&self[1],&t[1],&t[2]) { return true; }
        if line_intersect2(&self[0],&self[2],&t[0],&t[1]) { return true; }
        if line_intersect2(&self[0],&self[2],&t[0],&t[2]) { return true; }
        if line_intersect2(&self[0],&self[2],&t[1],&t[2]) { return true; }
        if line_intersect2(&self[1],&self[2],&t[0],&t[1]) { return true; }
        if line_intersect2(&self[1],&self[2],&t[0],&t[2]) { return true; }
        if line_intersect2(&self[1],&self[2],&t[1],&t[2]) { return true; }

        false
    }
}

fn line_intersect2(p11: &Point, p12: &Point, p21: &Point, p22: &Point) -> bool {
    let mut d = (p22.y - p21.y) * (p12.x - p11.x) - (p22.x - p21.x) * (p12.y - p11.y);

    let mut u = (p22.x - p21.x) * (p11.y - p21.y) - (p22.y - p21.y) * (p11.x - p21.x);

    let mut v = (p12.x - p11.x) * (p11.y - p21.y) - (p12.y - p11.y) * (p11.x - p21.x);

    if d < 0.0 {
        u *= -1f64;
        v *= -1f64;
        d *= -1f64;
    }

    0.01 <= u && u <= d - 0.01 && 0.01 <= v && v <= d - 0.01
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f64> for Point {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div for Point {
    type Output = Self;

    fn div(self, other: Point) -> Point {
        Point {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl Index<usize> for Triangle {
    type Output = Point;
    fn index<'a>(&'a self, i: usize) -> &'a Point {
        &self.points[i]
    }
}

impl IndexMut<usize> for Triangle {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut Point {
        &mut self.points[i]
    }
}
