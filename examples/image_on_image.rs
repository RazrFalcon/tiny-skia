use tiny_skia::*;

fn main() {
    let triangle = create_triangle();

    let mut pixmap = Pixmap::new(400, 400).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let now = std::time::Instant::now();

    let mut paint = PixmapPaint::default();
    paint.quality = FilterQuality::Bicubic;

    canvas.transform(1.2, 0.5, 0.5, 1.2, 0.0, 0.0);
    canvas.draw_pixmap(20, 20, triangle.as_ref(), &paint);

    println!("Rendered in {:.2}ms", now.elapsed().as_micros() as f64 / 1000.0);

    pixmap.save_png("image.png").unwrap();
}

fn create_triangle() -> Pixmap {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let mut pb = PathBuilder::new();
    pb.move_to(0.0, 200.0);
    pb.line_to(200.0, 200.0);
    pb.line_to(100.0, 0.0);
    pb.close();
    let path = pb.finish().unwrap();
    canvas.fill_path(&path, &paint, FillRule::Winding);

    let path = PathBuilder::from_rect(Rect::from_ltrb(0.0, 0.0, 200.0, 200.0).unwrap());
    let stroke = Stroke::default();
    paint.set_color_rgba8(200, 0, 0, 220);
    canvas.stroke_path(&path, &paint, &stroke); // TODO: stroke_rect

    pixmap
}
