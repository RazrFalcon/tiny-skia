use tiny_skia::*;

#[test]
fn fill_rect() {
    let pixmap = Pixmap::new(100, 100).unwrap();
    let mut canvas = Canvas::from(pixmap);

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.transform(1.2, 0.3, -0.7, 0.8, 12.0, 15.3);
    canvas.fill_rect(Rect::from_xywh(20.3, 10.4, 50.5, 30.2).unwrap(), &paint);

    let expected = Pixmap::load_png("tests/images/canvas/fill-rect.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn draw_pixmap() {
    // Tests that painting algorithm will switch `Bicubic`/`Bilinear` to `Nearest`.
    // Otherwise we will get a blurry image.

    // A pixmap with the bottom half filled with solid color.
    let sub_pixmap = {
        let mut canvas = Canvas::new(100, 100).unwrap();
        let mut paint = Paint::default();
        paint.set_color_rgba8(50, 127, 150, 200);
        let rect = Rect::from_xywh(0.0, 50.0, 100.0, 50.0).unwrap();
        canvas.fill_rect(rect, &paint);
        canvas.pixmap
    };

    let pixmap = Pixmap::new(200, 200).unwrap();
    let mut canvas = Canvas::from(pixmap);

    let mut paint = PixmapPaint::default();
    paint.quality = FilterQuality::Bicubic;

    canvas.draw_pixmap(20, 20, &sub_pixmap, &paint);

    let expected = Pixmap::load_png("tests/images/canvas/draw-pixmap.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn draw_pixmap_ts() {
    let triangle = {
        let mut canvas = Canvas::new(100, 100).unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(50, 127, 150, 200);
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.move_to(0.0, 100.0);
        pb.line_to(100.0, 100.0);
        pb.line_to(50.0, 0.0);
        pb.close();
        let path = pb.finish().unwrap();
        canvas.fill_path(&path, &paint, FillType::Winding);

        canvas.pixmap
    };

    let pixmap = Pixmap::new(200, 200).unwrap();
    let mut canvas = Canvas::from(pixmap);

    let mut paint = PixmapPaint::default();
    paint.quality = FilterQuality::Bicubic;

    canvas.transform(1.2, 0.5, 0.5, 1.2, 0.0, 0.0);
    canvas.draw_pixmap(5, 10, &triangle, &paint);

    let expected = Pixmap::load_png("tests/images/canvas/draw-pixmap-ts.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}

#[test]
fn draw_pixmap_opacity() {
    let triangle = {
        let mut canvas = Canvas::new(100, 100).unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(50, 127, 150, 200);
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.move_to(0.0, 100.0);
        pb.line_to(100.0, 100.0);
        pb.line_to(50.0, 0.0);
        pb.close();
        let path = pb.finish().unwrap();
        canvas.fill_path(&path, &paint, FillType::Winding);

        canvas.pixmap
    };

    let pixmap = Pixmap::new(200, 200).unwrap();
    let mut canvas = Canvas::from(pixmap);

    let mut paint = PixmapPaint::default();
    paint.quality = FilterQuality::Bicubic;
    paint.opacity = NormalizedF32::new_bounded(0.5);

    canvas.transform(1.2, 0.5, 0.5, 1.2, 0.0, 0.0);
    canvas.draw_pixmap(5, 10, &triangle, &paint);

    let expected = Pixmap::load_png("tests/images/canvas/draw-pixmap-opacity.png").unwrap();
    assert_eq!(canvas.pixmap, expected);
}
