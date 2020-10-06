use tiny_skia::*;

#[test]
fn horizontal_line() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(90.0, 10.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/empty.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn vertical_line() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(10.0, 90.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/empty.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn single_line() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(90.0, 90.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/empty.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn int_rect() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let rect = Rect::from_xywh(10.0, 15.0, 80.0, 70.0).unwrap();

    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/int-rect.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn float_rect() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let rect = Rect::from_xywh(10.3, 15.4, 80.5, 70.6).unwrap();

    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/float-rect.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn int_rect_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(10.0, 15.0, 80.0, 70.0).unwrap();

    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/int-rect-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn float_rect_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(10.3, 15.4, 80.5, 70.6).unwrap();

    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/float-rect-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn float_rect_aa_highp() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;
    paint.force_hq_pipeline = true;

    let rect = Rect::from_xywh(10.3, 15.4, 80.5, 70.6).unwrap();

    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/float-rect-aa-highp.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn tiny_float_rect() {
    let mut canvas = Canvas::new(3, 3).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let rect = Rect::from_xywh(1.3, 1.4, 0.5, 0.6).unwrap();
    canvas.fill_rect(&rect, &paint);

    assert_eq!(
        canvas.pixmap.pixels(),
        &[
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),

            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(50, 127, 150, 200).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),

            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
        ]
    );
}

#[test]
fn tiny_float_rect_aa() {
    let mut canvas = Canvas::new(3, 3).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(1.3, 1.4, 0.5, 0.6).unwrap();
    canvas.fill_rect(&rect, &paint);

    assert_eq!(
        canvas.pixmap.pixels(),
        &[
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),

            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(51, 128, 153, 60).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),

            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
            ColorU8::from_rgba(0, 0, 0, 0).premultiply(),
        ]
    );
}

#[test]
fn float_rect_clip_top_left_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(-10.3, -20.4, 100.5, 70.2).unwrap();
    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/float-rect-clip-top-left-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn float_rect_clip_top_right_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(60.3, -20.4, 100.5, 70.2).unwrap();
    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/float-rect-clip-top-right-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn float_rect_clip_bottom_right_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(60.3, 40.4, 100.5, 70.2).unwrap();
    canvas.fill_rect(&rect, &paint);

    let expected = Pixmap::load_png("tests/images/fill/float-rect-clip-bottom-right-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn open_polygon() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(75.160671, 88.756136);
    pb.line_to(24.797274, 88.734053);
    pb.line_to( 9.255130, 40.828792);
    pb.line_to(50.012955, 11.243795);
    pb.line_to(90.744819, 40.864522);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/polygon.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

// Must be the same a open.
#[test]
fn closed_polygon() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(75.160671, 88.756136);
    pb.line_to(24.797274, 88.734053);
    pb.line_to( 9.255130, 40.828792);
    pb.line_to(50.012955, 11.243795);
    pb.line_to(90.744819, 40.864522);
    pb.close(); // the only difference
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/polygon.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn winding_star() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(50.0,  7.5);
    pb.line_to(75.0, 87.5);
    pb.line_to(10.0, 37.5);
    pb.line_to(90.0, 37.5);
    pb.line_to(25.0, 87.5);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/winding-star.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn even_odd_star() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(50.0,  7.5);
    pb.line_to(75.0, 87.5);
    pb.line_to(10.0, 37.5);
    pb.line_to(90.0, 37.5);
    pb.line_to(25.0, 87.5);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::EvenOdd);

    let expected = Pixmap::load_png("tests/images/fill/even-odd-star.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn quad_curve() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 15.0);
    pb.quad_to(95.0, 35.0, 75.0, 90.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::EvenOdd);

    let expected = Pixmap::load_png("tests/images/fill/quad.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn cubic_curve() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 15.0);
    pb.cubic_to(95.0, 35.0, 0.0, 75.0, 75.0, 90.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::EvenOdd);

    let expected = Pixmap::load_png("tests/images/fill/cubic.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn memset2d() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 255); // Must be opaque to trigger memset2d.

    let path = PathBuilder::from_bounds(Bounds::from_ltrb(10.0, 10.0, 90.0, 90.0).unwrap());
    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/memset2d.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

// Make sure we do not write past pixmap memory.
#[test]
fn memset2d_out_of_bounds() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 255); // Must be opaque to trigger memset2d.

    let path = PathBuilder::from_bounds(Bounds::from_ltrb(50.0, 50.0, 120.0, 120.0).unwrap());
    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/memset2d-2.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn fill_pixmap() {
    let mut canvas = Canvas::new(10, 10).unwrap();
    let c = Color::from_rgba8(50, 100, 150, 200);
    canvas.fill_canvas(c);
    assert_eq!(canvas.pixmap.pixel(1, 1).unwrap(), c.premultiply().to_color_u8());
}

// Not sure how to properly test anti-aliasing,
// so for now simply check that it actually applied.
#[test]
fn fill_aa() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let mut pb = PathBuilder::new();
    pb.move_to(50.0,  7.5);
    pb.line_to(75.0, 87.5);
    pb.line_to(10.0, 37.5);
    pb.line_to(90.0, 37.5);
    pb.line_to(25.0, 87.5);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::EvenOdd);

    let expected = Pixmap::load_png("tests/images/fill/star-aa.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn overflow_in_walk_edges_1() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.cubic_to(39.0, 163.0, 117.0, 61.0, 130.0, 70.0);
    let path = pb.finish().unwrap();

    // Must not panic.
    canvas.fill_path(&path, &paint, FillType::Winding);
}

#[test]
fn clip_line_1() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(50.0, -15.0);
    pb.line_to(-15.0, 50.0);
    pb.line_to(50.0, 115.0);
    pb.line_to(115.0, 50.0);
    pb.close();
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/clip-line-1.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn clip_line_2() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    // This strange path forces `line_clipper::clip` to return an empty array.
    // And we're checking that this case is handled correctly.
    let mut pb = PathBuilder::new();
    pb.move_to(0.0, -1.0);
    pb.line_to(50.0, 0.0);
    pb.line_to(0.0, 50.0);
    pb.close();
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/clip-line-2.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn clip_quad() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 85.0);
    pb.quad_to(150.0, 150.0, 85.0, 15.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/clip-quad.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn clip_cubic_1() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    // `line_clipper::clip` produces 2 points for this path.
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 50.0);
    pb.cubic_to(0.0, 175.0, 195.0, 70.0, 75.0, 20.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/clip-cubic-1.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn clip_cubic_2() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    // `line_clipper::clip` produces 3 points for this path.
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 50.0);
    pb.cubic_to(10.0, 40.0, 90.0, 120.0, 125.0, 20.0);
    let path = pb.finish().unwrap();

    canvas.fill_path(&path, &paint, FillType::Winding);

    let expected = Pixmap::load_png("tests/images/fill/clip-cubic-2.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn aa_endless_loop() {
    let mut canvas = Canvas::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.anti_alias = true;

    // This path was causing an endless loop before.
    let mut pb = PathBuilder::new();
    pb.move_to(2.1537175, 11.560721);
    pb.quad_to(1.9999998, 10.787931, 2.0, 10.0);
    let path = pb.finish().unwrap();

    // Must not loop.
    canvas.fill_path(&path, &paint, FillType::Winding);
}
