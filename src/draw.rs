use crate::types::Stroke;

pub fn render_stroke(pixmap: &mut tiny_skia::PixmapMut, stroke: &Stroke) {
    if stroke.points.len() < 2 {
        return;
    }
    let mut pb = tiny_skia::PathBuilder::new();
    let first = &stroke.points[0];
    pb.move_to(first.x, first.y);
    for p in &stroke.points[1..] {
        pb.line_to(p.x, p.y);
    }
    if let Some(path) = pb.finish() {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(stroke.color);
        let stroke_opts = tiny_skia::Stroke {
            width: stroke.thickness,
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
