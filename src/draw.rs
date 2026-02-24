use crate::types::Shape;

pub fn render_shape(pixmap: &mut tiny_skia::PixmapMut, shape: &Shape) {
    let mut pb = tiny_skia::PathBuilder::new();
    let (color, thickness) = match shape {
        Shape::Freehand {
            points,
            color,
            thickness,
        } => {
            if points.len() < 2 {
                return;
            }
            pb.move_to(points[0].x, points[0].y);
            for p in &points[1..] {
                pb.line_to(p.x, p.y);
            }
            (*color, *thickness)
        }
        Shape::Rectangle {
            start,
            end,
            color,
            thickness,
        } => {
            let min_x = start.x.min(end.x);
            let min_y = start.y.min(end.y);
            let w = (start.x - end.x).abs();
            let h = (start.y - end.y).abs();

            pb.move_to(min_x, min_y);
            pb.line_to(min_x + w, min_y);
            pb.line_to(min_x + w, min_y + h);
            pb.line_to(min_x, min_y + h);
            pb.close();

            (*color, *thickness)
        }
        Shape::Arrow {
            start,
            end,
            color,
            thickness,
        } => {
            // Main line
            pb.move_to(start.x, start.y);
            pb.line_to(end.x, end.y);

            // Arrowhead
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let len = (dx * dx + dy * dy).sqrt();

            if len > 0.1 {
                let head_len = 15.0 + (*thickness * 1.5); // Adjust size as needed
                let head_angle = std::f32::consts::PI / 6.0; // 30 degrees

                let angle = dy.atan2(dx);

                let x1 = end.x - head_len * (angle - head_angle).cos();
                let y1 = end.y - head_len * (angle - head_angle).sin();

                let x2 = end.x - head_len * (angle + head_angle).cos();
                let y2 = end.y - head_len * (angle + head_angle).sin();

                pb.move_to(end.x, end.y);
                pb.line_to(x1, y1);
                pb.move_to(end.x, end.y);
                pb.line_to(x2, y2);
            }

            (*color, *thickness)
        }
    };

    if let Some(path) = pb.finish() {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(color);
        let stroke_opts = tiny_skia::Stroke {
            width: thickness,
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            ..Default::default()
        };
        pixmap.stroke_path(
            &path,
            &paint,
            &stroke_opts,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}
