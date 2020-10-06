use tiny_skia::*;

// This example demonstrates thin paths rendering.

fn main() {
    let pixmap = Pixmap::new(500, 500).unwrap();
    let mut canvas = Canvas::from(pixmap);

    let now = std::time::Instant::now();

    let mut pb = PathBuilder::new();
    pb.move_to(50.0, 100.0);
    pb.cubic_to(130.0, 20.0, 390.0, 120.0, 450.0, 30.0);
    let path = pb.finish().unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    for i in 0..20 {
        let mut stroke = Stroke::default();
        stroke.width = 2.0 - (i as f32 / 10.0);
        canvas.stroke_path(&path, &paint, stroke);
        canvas.translate(0.0, 20.0);
    }

    println!("Rendered in {:.2}ms", now.elapsed().as_micros() as f64 / 1000.0);

    canvas.pixmap.save_png("image.png").unwrap();
}
