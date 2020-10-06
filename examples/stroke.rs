use tiny_skia::*;

fn main() {
    let mut canvas = Canvas::new(500, 500).unwrap();

    let now = std::time::Instant::now();

    let mut paint = Paint::default();
    paint.set_color_rgba8(0xDD, 0, 0, 0xAA);
    paint.anti_alias = true;

    let path = {
        let mut pb = PathBuilder::new();
        pb.move_to(45.0, 360.0);
        pb.line_to(220.0, 260.0);
        pb.line_to(455.0, 260.0);
        pb.line_to(280.0, 360.0);
        pb.close();
        pb.finish().unwrap()
    };

    let mut stroke = StrokeProps::default();
    stroke.width = 4.0;

    canvas.stroke_path(&path, &paint, stroke);

    println!("Rendered in {:.2}ms", now.elapsed().as_micros() as f64 / 1000.0);

    canvas.pixmap.save_png("image.png").unwrap();
}
