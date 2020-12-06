use tiny_skia::*;

#[test]
fn clone_rect_1() {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.fill_path(
        &PathBuilder::from_circle(100.0, 100.0, 80.0).unwrap(),
        &paint,
        FillRule::Winding,
    );

    let part = pixmap.as_ref().clone_rect(IntRect::from_xywh(10, 15, 80, 90).unwrap()).unwrap();

    let expected = Pixmap::load_png("tests/images/pixmap/clone-rect-1.png").unwrap();
    assert_eq!(part, expected);
}

#[test]
fn clone_rect_2() {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.fill_path(
        &PathBuilder::from_circle(100.0, 100.0, 80.0).unwrap(),
        &paint,
        FillRule::Winding,
    );

    let part = pixmap.as_ref().clone_rect(IntRect::from_xywh(130, 120, 80, 90).unwrap()).unwrap();

    let expected = Pixmap::load_png("tests/images/pixmap/clone-rect-2.png").unwrap();
    assert_eq!(part, expected);
}

#[test]
fn clone_rect_out_of_bound() {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.fill_path(
        &PathBuilder::from_circle(100.0, 100.0, 80.0).unwrap(),
        &paint,
        FillRule::Winding,
    );

    assert!(pixmap.as_ref().clone_rect(IntRect::from_xywh(250, 15, 80, 90).unwrap()).is_none());
    assert!(pixmap.as_ref().clone_rect(IntRect::from_xywh(10, 250, 80, 90).unwrap()).is_none());
    assert!(pixmap.as_ref().clone_rect(IntRect::from_xywh(10, -250, 80, 90).unwrap()).is_none());
}

#[test]
fn fill() {
    let c = Color::from_rgba8(50, 100, 150, 200);
    let mut pixmap = Pixmap::new(10, 10).unwrap();
    pixmap.fill(c);
    assert_eq!(pixmap.pixel(1, 1).unwrap(), c.premultiply().to_color_u8());
}
