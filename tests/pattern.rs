use tiny_skia::*;

fn crate_triangle() -> Pixmap {
    let mut canvas = Canvas::new(20, 20).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let mut pb = PathBuilder::new();
    pb.move_to(0.0, 20.0);
    pb.line_to(20.0, 20.0);
    pb.line_to(10.0, 0.0);
    pb.close();
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillRule::Winding);

    canvas.pixmap
}

#[test]
fn pad_nearest() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Pad,
        FilterQuality::Nearest,
        1.0,
        Transform::identity(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/pad-nearest.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn repeat_nearest() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Repeat,
        FilterQuality::Nearest,
        1.0,
        Transform::identity(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/repeat-nearest.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn reflect_nearest() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Reflect,
        FilterQuality::Nearest,
        1.0,
        Transform::identity(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/reflect-nearest.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

// We have to test tile mode for bilinear/bicubic separately,
// because they're using a different algorithm from nearest.
#[test]
fn pad_bicubic() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Pad,
        FilterQuality::Bicubic,
        1.0,
        // Transform must be set, otherwise we will fallback to Nearest.
        Transform::from_row(1.1, 0.3, 0.0, 1.4, 0.0, 0.0).unwrap(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/pad-bicubic.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn repeat_bicubic() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Repeat,
        FilterQuality::Bicubic,
        1.0,
        // Transform must be set, otherwise we will fallback to Nearest.
        Transform::from_row(1.1, 0.3, 0.0, 1.4, 0.0, 0.0).unwrap(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/repeat-bicubic.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn reflect_bicubic() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Reflect,
        FilterQuality::Bicubic,
        1.0,
        // Transform must be set, otherwise we will fallback to Nearest.
        Transform::from_row(1.1, 0.3, 0.0, 1.4, 0.0, 0.0).unwrap(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/reflect-bicubic.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn filter_nearest_no_ts() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Repeat,
        FilterQuality::Nearest,
        1.0,
        Transform::identity(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/filter-nearest-no-ts.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn filter_nearest() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Repeat,
        FilterQuality::Nearest,
        1.0,
        Transform::from_row(1.5, 0.0, -0.4, -0.8, 5.0, 1.0).unwrap(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/filter-nearest.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn filter_bilinear() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Repeat,
        FilterQuality::Bilinear,
        1.0,
        Transform::from_row(1.5, 0.0, -0.4, -0.8, 5.0, 1.0).unwrap(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/filter-bilinear.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn filter_bicubic() {
    let mut canvas = Canvas::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.shader = Pattern::new(
        &triangle,
        SpreadMode::Repeat,
        FilterQuality::Bicubic,
        1.0,
        Transform::from_row(1.5, 0.0, -0.4, -0.8, 5.0, 1.0).unwrap(),
    );

    let path = PathBuilder::from_rect(Rect::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    canvas.fill_path(&path, &paint, FillRule::Winding);

    let expected = Pixmap::load_png("tests/images/pattern/filter-bicubic.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}
