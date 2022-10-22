use tiny_skia::*;

fn main() {
    let clip_path = {
        let mut pb = PathBuilder::new();
        pb.push_circle(250.0, 250.0, 200.0);
        pb.push_circle(250.0, 250.0, 100.0);
        pb.finish().unwrap()
    };

    let clip_path = clip_path
        .transform(Transform::from_row(1.0, -0.3, 0.0, 1.0, 0.0, 75.0))
        .unwrap();

    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(500, 500, &clip_path, FillRule::EvenOdd, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pixmap = Pixmap::new(500, 500).unwrap();
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, 500.0, 500.0).unwrap(),
        &paint,
        Transform::identity(),
        Some(&clip_mask),
    );
    pixmap.save_png("image.png").unwrap();
}
