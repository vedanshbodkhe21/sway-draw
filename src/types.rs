#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn union(&self, other: &Rect) -> Rect {
        let max_x = std::cmp::max(self.x + self.w as i32, other.x + other.w as i32);
        let max_y = std::cmp::max(self.y + self.h as i32, other.y + other.h as i32);
        let min_x = std::cmp::min(self.x, other.x);
        let min_y = std::cmp::min(self.y, other.y);

        Rect {
            x: min_x,
            y: min_y,
            w: (max_x - min_x) as u32,
            h: (max_y - min_y) as u32,
        }
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let max_x = std::cmp::min(self.x + self.w as i32, other.x + other.w as i32);
        let max_y = std::cmp::min(self.y + self.h as i32, other.y + other.h as i32);
        let min_x = std::cmp::max(self.x, other.x);
        let min_y = std::cmp::max(self.y, other.y);

        if min_x < max_x && min_y < max_y {
            Some(Rect {
                x: min_x,
                y: min_y,
                w: (max_x - min_x) as u32,
                h: (max_y - min_y) as u32,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Rectangle,
    Arrow,
    Freehand,
}

#[derive(Clone, Debug)]
pub enum Shape {
    Freehand {
        points: Vec<Point>,
        color: tiny_skia::Color,
        thickness: f32,
    },
    Rectangle {
        start: Point,
        end: Point,
        color: tiny_skia::Color,
        thickness: f32,
    },
    Arrow {
        start: Point,
        end: Point,
        color: tiny_skia::Color,
        thickness: f32,
    },
}

impl Shape {
    pub fn bounding_box(&self) -> Option<Rect> {
        let (mut min_x, mut min_y, mut max_x, mut max_y, thickness) = match self {
            Shape::Freehand {
                points, thickness, ..
            } => {
                if points.is_empty() {
                    return None;
                }
                let mut min_x = points[0].x;
                let mut min_y = points[0].y;
                let mut max_x = points[0].x;
                let mut max_y = points[0].y;

                for p in &points[1..] {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
                (min_x, min_y, max_x, max_y, *thickness)
            }
            Shape::Rectangle {
                start,
                end,
                thickness,
                ..
            }
            | Shape::Arrow {
                start,
                end,
                thickness,
                ..
            } => {
                let min_x = start.x.min(end.x);
                let min_y = start.y.min(end.y);
                let max_x = start.x.max(end.x);
                let max_y = start.y.max(end.y);
                (min_x, min_y, max_x, max_y, *thickness)
            }
        };

        // Pad by thickness
        let pad = thickness / 2.0 + 2.0; // slight extra padding for anti-aliasing edge cases

        // For Arrow, pad a bit more to account for arrowhead which can extend slightly beyond bounds
        let pad = match self {
            Shape::Arrow { thickness, .. } => pad + (*thickness * 3.0),
            _ => pad,
        };

        min_x -= pad;
        min_y -= pad;
        max_x += pad;
        max_y += pad;

        Some(Rect {
            x: min_x.floor() as i32,
            y: min_y.floor() as i32,
            w: (max_x.ceil() - min_x.floor()) as u32,
            h: (max_y.ceil() - min_y.floor()) as u32,
        })
    }
}
