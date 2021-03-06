use tiny_skia::*;

#[test]
fn rect() {
    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(100, 100, &clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let rect = Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap();
    pixmap.fill_rect(rect, &paint, Transform::identity(), Some(&clip_mask));

    let expected = Pixmap::load_png("tests/images/clip/rect.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn rect_aa() {
    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.5, 10.0, 80.0, 80.5).unwrap());
    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(100, 100, &clip_path, FillRule::Winding, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pixmap = Pixmap::new(100, 100).unwrap();
    let rect = Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap();
    pixmap.fill_rect(rect, &paint, Transform::identity(), Some(&clip_mask));

    let expected = Pixmap::load_png("tests/images/clip/rect-aa.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn rect_ts() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    let clip_path = clip_path.transform(Transform::from_row(1.0, -0.3, 0.0, 1.0, 0.0, 15.0)).unwrap();

    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(100, 100, &clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let rect = Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap();
    pixmap.fill_rect(rect, &paint, Transform::identity(), Some(&clip_mask));

    let expected = Pixmap::load_png("tests/images/clip/rect-ts.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn circle_bottom_right_aa() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_circle(100.0, 100.0, 50.0).unwrap();
    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(100, 100, &clip_path, FillRule::Winding, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let rect = Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap();
    pixmap.fill_rect(rect, &paint, Transform::identity(), Some(&clip_mask));

    let expected = Pixmap::load_png("tests/images/clip/circle-bottom-right-aa.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn stroke() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(100, 100, &clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut stroke = Stroke::default();
    stroke.width = 10.0;

    let path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), Some(&clip_mask));

    let expected = Pixmap::load_png("tests/images/clip/stroke.png").unwrap();
    assert_eq!(pixmap, expected);
}

// Make sure we're clipping only source and not source and destination
#[test]
fn skip_dest() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    pixmap.fill_path(
        &PathBuilder::from_rect(Rect::from_xywh(5.0, 5.0, 60.0, 60.0).unwrap()),
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    let mut pixmap2 = Pixmap::new(200, 200).unwrap();
    pixmap2.as_mut().fill_path(
        &PathBuilder::from_rect(Rect::from_xywh(35.0, 35.0, 60.0, 60.0).unwrap()),
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    let clip_path = PathBuilder::from_rect(Rect::from_xywh(40.0, 40.0, 40.0, 40.0).unwrap());
    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(100, 100, &clip_path, FillRule::Winding, true);

    pixmap.draw_pixmap(0, 0, pixmap2.as_ref(), &PixmapPaint::default(),
                                Transform::identity(), Some(&clip_mask));

    let expected = Pixmap::load_png("tests/images/clip/skip-dest.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn intersect_aa() {
    let circle1 = PathBuilder::from_circle(75.0, 75.0, 50.0).unwrap();
    let circle2 = PathBuilder::from_circle(125.0, 125.0, 50.0).unwrap();

    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(200, 200, &circle1, FillRule::Winding, true);
    clip_mask.intersect_path(&circle2, FillRule::Winding, true);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let mut pixmap = Pixmap::new(200, 200).unwrap();
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, 200.0, 200.0).unwrap(),
        &paint,
        Transform::identity(),
        Some(&clip_mask),
    );

    let expected = Pixmap::load_png("tests/images/clip/intersect-aa.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn ignore_memset() {
    let clip_path = PathBuilder::from_rect(Rect::from_xywh(10.0, 10.0, 80.0, 80.0).unwrap());

    let mut clip_mask = ClipMask::new();
    clip_mask.set_path(100, 100, &clip_path, FillRule::Winding, false);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 255);

    let mut pixmap = Pixmap::new(100, 100).unwrap();
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
        &paint,
        Transform::identity(),
        Some(&clip_mask),
    );

    let expected = Pixmap::load_png("tests/images/clip/ignore-memset.png").unwrap();
    assert_eq!(pixmap, expected);
}
