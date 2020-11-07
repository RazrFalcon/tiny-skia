use tiny_skia::*;

#[test]
fn rect() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.set_clip_path(&clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/rect.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn rect_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.5, 10.0, 80.0, 80.5).unwrap());
    canvas.set_clip_path(&clip_path, FillRule::Winding, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/rect-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn rect_ts() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.set_transform(Transform::from_row(1.0, -0.3, 0.0, 1.0, 0.0, 15.0).unwrap());
    canvas.set_clip_path(&clip_path, FillRule::Winding, false);
    canvas.reset_transform();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/rect-ts.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn circle_bottom_right_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_circle(100.0, 100.0, 50.0).unwrap();
    canvas.set_clip_path(&clip_path, FillRule::Winding, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/circle-bottom-right-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn stroke() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.set_clip_path(&clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut stroke = Stroke::default();
    stroke.width = 10.0;

    let path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.stroke_path(&path, &paint, &stroke);

    let expected = Pixmap::load_png("tests/images/clip/stroke.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}
