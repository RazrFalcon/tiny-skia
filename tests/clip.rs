use tiny_skia::*;

#[test]
fn rect() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.set_clip_path(&clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/rect.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn rect_aa() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.5, 10.0, 80.0, 80.5).unwrap());
    canvas.set_clip_path(&clip_path, FillRule::Winding, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/rect-aa.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn rect_ts() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.set_transform(Transform::from_row(1.0, -0.3, 0.0, 1.0, 0.0, 15.0));
    canvas.set_clip_path(&clip_path, FillRule::Winding, false);
    canvas.reset_transform();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/rect-ts.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn circle_bottom_right_aa() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let clip_path = PathBuilder::from_circle(100.0, 100.0, 50.0).unwrap();
    canvas.set_clip_path(&clip_path, FillRule::Winding, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_rect(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/clip/circle-bottom-right-aa.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn stroke() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.set_clip_path(&clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut stroke = Stroke::default();
    stroke.width = 10.0;

    let path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    canvas.stroke_path(&path, &paint, &stroke);

    let expected = Pixmap::load_png("tests/images/clip/stroke.png").unwrap();
    assert_eq!(pixmap, expected);
}

// Make sure we're clipping only source and not source and destination
#[test]
fn skip_dest() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    canvas.fill_path(
        &PathBuilder::from_rect(Rect::from_xywh(5.0, 5.0, 60.0, 60.0).unwrap()),
        &paint,
        FillRule::Winding,
    );

    let mut pixmap2 = Pixmap::new(200, 200).unwrap();
    let mut canvas2 = Canvas::from(pixmap2.as_mut());
    canvas2.fill_path(
        &PathBuilder::from_rect(Rect::from_xywh(35.0, 35.0, 60.0, 60.0).unwrap()),
        &paint,
        FillRule::Winding,
    );

    canvas.set_clip_rect(Rect::from_xywh(40.0, 40.0, 40.0, 40.0).unwrap(), true);
    canvas.draw_pixmap(0, 0, pixmap2.as_ref(), &PixmapPaint::default());

    let expected = Pixmap::load_png("tests/images/clip/skip-dest.png").unwrap();
    assert_eq!(pixmap, expected);
}
