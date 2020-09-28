use tiny_skia::*;

#[test]
fn fill_rect() {
    let pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.transform = Transform::from_row(1.2, 0.3, -0.7, 0.8, 12.0, 15.3).unwrap();
    canvas.fill_rect(&Rect::from_xywh(20.3, 10.4, 50.5, 30.2).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/canvas/fill-rect.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}
