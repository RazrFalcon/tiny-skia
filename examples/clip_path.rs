use tiny_skia::*;

fn main() {
    let mut pixmap = Pixmap::new(500, 500).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let now = std::time::Instant::now();

    let clip_path = {
        let mut pb = PathBuilder::new();
        pb.push_circle(250.0, 250.0, 200.0);
        pb.push_circle(250.0, 250.0, 100.0);
        pb.finish().unwrap()
    };

    // Skew the circle before clipping.
    // Clip path is processed immediately and affected by the current transform.
    canvas.set_transform(Transform::from_row(1.0, -0.3, 0.0, 1.0, 0.0, 75.0));
    canvas.set_clip_path(&clip_path, FillRule::EvenOdd, true);

    // Reset the transform, so the rectangle would be rendered as is,
    // while clip path will stay transformed.
    canvas.reset_transform();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 500.0, 500.0).unwrap(), &paint);

    println!("Rendered in {:.2}ms", now.elapsed().as_micros() as f64 / 1000.0);

    pixmap.save_png("image.png").unwrap();
}
