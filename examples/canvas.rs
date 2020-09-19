use tiny_skia::*;

fn main() {
    let pixmap = Pixmap::new(1000, 1000).unwrap();
    let mut canvas = Canvas::from(pixmap);

    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.shader = LinearGradient::new(
        Point::from_xy(100.0, 40.0),
        Point::from_xy(210.0, 210.0),
        vec![
            GradientStop::new(0.0, Color::from_rgba8(50, 127, 150, 200)),
            GradientStop::new(1.0, Color::from_rgba8(220, 140, 75, 180)),
        ],
        SpreadMode::Pad,
        Transform::default(),
    ).unwrap();

    let mut pb = PathBuilder::new();
    pb.move_to(100.0, 40.0);
    pb.line_to(210.0, 40.0);
    pb.line_to(210.0, 150.0);
    pb.line_to(100.0, 150.0);
    pb.close();
    let path = pb.finish().unwrap();

    let mut stroke = StrokeProps::default();
    stroke.width = 20.0;

    let ts = Transform::from_row(1.5, 0.3, 1.7, 3.5, -130.0, 50.0).unwrap();
    canvas.transform = canvas.transform.post_concat(&ts).unwrap();

    canvas.stroke_path(&path, &paint, stroke);

    canvas.transform = canvas.transform.post_scale(1.0, -1.0).unwrap();
    canvas.transform = canvas.transform.post_translate(480.0, 858.0).unwrap();
    canvas.stroke_path(&path, &paint, stroke);

    canvas.pixmap.save_png("image.png").unwrap();
}
