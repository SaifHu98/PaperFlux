use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WritingDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Baseline {
    pub y: f32,
    pub x_start: f32,
    pub x_end: f32,
    pub angle: f32, // in radians
}

impl Baseline {
    pub fn new(y: f32, x_start: f32, x_end: f32) -> Self {
        Self {
            y,
            x_start,
            x_end,
            angle: 0.0,
        }
    }

    pub fn width(&self) -> f32 {
        (self.x_end - self.x_start).abs()
    }

    pub fn is_co_linear(&self, other: &Baseline, tolerance: f32) -> bool {
        (self.y - other.y).abs() <= tolerance && (self.angle - other.angle).abs() <= 0.05
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p1.x - p2.x).abs();
        let height = (p1.y - p2.y).abs();
        Self::new(x, y, width, height)
    }

    pub fn x_min(&self) -> f32 {
        self.x
    }

    pub fn x_max(&self) -> f32 {
        self.x + self.width
    }

    pub fn y_min(&self) -> f32 {
        self.y
    }

    pub fn y_max(&self) -> f32 {
        self.y + self.height
    }

    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }

    pub fn center(&self) -> Point {
        Point::new(self.center_x(), self.center_y())
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.x_min() && p.x <= self.x_max() && p.y >= self.y_min() && p.y <= self.y_max()
    }

    pub fn contains_rect(&self, other: &Rect) -> bool {
        self.x_min() <= other.x_min()
            && self.x_max() >= other.x_max()
            && self.y_min() <= other.y_min()
            && self.y_max() >= other.y_max()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x_min() <= other.x_max()
            && self.x_max() >= other.x_min()
            && self.y_min() <= other.y_max()
            && self.y_max() >= other.y_min()
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }
        let min_x = self.x_min().max(other.x_min());
        let min_y = self.y_min().max(other.y_min());
        let max_x = self.x_max().min(other.x_max());
        let max_y = self.y_max().min(other.y_max());
        Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let min_x = self.x_min().min(other.x_min());
        let min_y = self.y_min().min(other.y_min());
        let max_x = self.x_max().max(other.x_max());
        let max_y = self.y_max().max(other.y_max());
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn horizontal_overlap(&self, other: &Rect) -> f32 {
        let min_max = self.x_max().min(other.x_max());
        let max_min = self.x_min().max(other.x_min());
        (min_max - max_min).max(0.0)
    }

    pub fn vertical_overlap(&self, other: &Rect) -> f32 {
        let min_max = self.y_max().min(other.y_max());
        let max_min = self.y_min().max(other.y_min());
        (min_max - max_min).max(0.0)
    }

    pub fn intersection_over_union(&self, other: &Rect) -> f32 {
        let inter_area = self.intersection(other).map(|r| r.area()).unwrap_or(0.0);
        let union_area = self.area() + other.area() - inter_area;
        if union_area > 0.0 {
            inter_area / union_area
        } else {
            0.0
        }
    }

    pub fn distance_to(&self, other: &Rect) -> f32 {
        let dx = (self.x_min() - other.x_max())
            .max(other.x_min() - self.x_max())
            .max(0.0);
        let dy = (self.y_min() - other.y_max())
            .max(other.y_min() - self.y_max())
            .max(0.0);
        (dx * dx + dy * dy).sqrt()
    }
}

pub type BoundingBox = Rect;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn multiply(&self, other: &Matrix) -> Matrix {
        Matrix {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            e: self.e * other.a + self.f * other.c + other.e,
            f: self.e * other.b + self.f * other.d + other.f,
        }
    }

    pub fn transform_point(&self, p: Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.e,
            y: self.b * p.x + self.d * p.y + self.f,
        }
    }

    pub fn scale_x(&self) -> f32 {
        (self.a * self.a + self.b * self.b).sqrt()
    }

    pub fn scale_y(&self) -> f32 {
        (self.c * self.c + self.d * self.d).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn is_dark(&self) -> bool {
        (0.299 * self.r + 0.587 * self.g + 0.114 * self.b) < 0.5
    }

    pub fn to_hex(&self) -> String {
        let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}
