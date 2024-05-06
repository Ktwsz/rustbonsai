use std::ops::{Add, Sub, Mul, Div};
use std::cmp::{PartialEq, Eq};

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64, 
    pub y: f64,
}

impl Point {
    pub fn from_floats(x: f64, y: f64) -> Self {
        Point {x, y}
    }

    pub fn normalize(&mut self, min_p: &Point, max_p: &Point, bounds: (u32, u32)) {
        let normal_x = max_p.x - min_p.x + 1.0;
        let normal_y = max_p.y - min_p.y + 1.0;
        let normal_p = Point::from_floats(normal_x, normal_y);

        let self_copy = *self;

        let normalized = (self_copy - *min_p) / normal_p;

        self.x = normalized.x * bounds.0 as f64;
        self.y = normalized.y * bounds.1 as f64;
    }
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

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        f64::abs(self.x - other.x) <= 0.000001 &&
            f64::abs(self.y - other.y) <= 0.000001  
    }
}

impl Eq for Point {}
