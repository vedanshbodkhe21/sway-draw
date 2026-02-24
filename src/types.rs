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

#[derive(Clone, Debug)]
pub struct Stroke {
    pub points: Vec<Point>,
    pub color: tiny_skia::Color,
    pub thickness: f32,
}

impl Stroke {
    pub fn bounding_box(&self) -> Option<Rect> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = self.points[0].x;
        let mut min_y = self.points[0].y;
        let mut max_x = self.points[0].x;
        let mut max_y = self.points[0].y;

        for p in &self.points[1..] {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        // Pad by thickness
        let pad = self.thickness / 2.0 + 2.0; // slight extra padding for anti-aliasing edge cases
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
