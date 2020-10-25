use tiny_skia::*;

// Based on https://fiddle.skia.org/c/@compose_path

fn main() {
    let mut canvas = Canvas::new(500, 500).unwrap();

    let now = std::time::Instant::now();

    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 127, 0, 200);
    paint.anti_alias = true;

    let path = {
        let mut pb = PathBuilder::new();
        const RADIUS: f32 = 250.0;
        const CENTER: f32 = 250.0;
        pb.move_to(CENTER + RADIUS, CENTER);
        for i in 1..8 {
            let a = 2.6927937 * i as f32;
            pb.line_to(CENTER + RADIUS * a.cos(), CENTER + RADIUS * a.sin());
        }
        pb.finish().unwrap()
    };

    let mut stroke = Stroke::default();
    stroke.width = 6.0;
    stroke.line_cap = LineCap::Round;
    stroke.dash = StrokeDash::new(vec![20.0, 40.0], 0.0);

    canvas.stroke_path(&path, &paint, &stroke);

    println!("Rendered in {:.2}ms", now.elapsed().as_micros() as f64 / 1000.0);

    canvas.pixmap.save_png("image.png").unwrap();
}
