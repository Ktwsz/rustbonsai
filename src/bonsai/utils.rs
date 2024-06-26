use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64, 
    pub y: f64,
}

impl Point {
    pub fn from_floats(x: f64, y: f64) -> Self {
        Point {x, y}
    }

    pub fn from_phi(phi: f64) -> Self {
        Point {
            x: f64::cos(phi),
            y: f64::sin(phi),
        }
    }

    pub fn norm2(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(&mut self, min_p: &Point, max_p: &Point, bounds: (u16, u16)) {
        let normal_x = max_p.x - min_p.x + 1.0;
        let normal_y = max_p.y - min_p.y + 1.0;
        let normal_p = Point::from_floats(normal_x, normal_y);

        let self_copy = *self;

        let normalized = (self_copy - *min_p) / normal_p;

        self.x = normalized.x * bounds.0 as f64;
        self.y = normalized.y * bounds.1 as f64;
    }

}

pub fn linear_interpolate(start: &Point, end: &Point, dt: f64) -> Point {
    *start + (*end - *start) * dt
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
