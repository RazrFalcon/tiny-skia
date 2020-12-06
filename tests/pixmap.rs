use tiny_skia::*;

#[test]
fn clone_rect_1() {
    let mut canvas = Canvas::new(200, 200).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.fill_path(
        &PathBuilder::from_circle(100.0, 100.0, 80.0).unwrap(),
        &paint,
        FillRule::Winding,
    );

    let part = canvas.pixmap.clone_rect(IntRect::from_xywh(10, 15, 80, 90).unwrap()).unwrap();

    let expected = Pixmap::load_png("tests/images/pixmap/clone-rect-1.png").unwrap();
    assert_eq!(part, expected);
}

#[test]
fn clone_rect_2() {
    let mut canvas = Canvas::new(200, 200).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.fill_path(
        &PathBuilder::from_circle(100.0, 100.0, 80.0).unwrap(),
        &paint,
        FillRule::Winding,
    );

    let part = canvas.pixmap.clone_rect(IntRect::from_xywh(130, 120, 80, 90).unwrap()).unwrap();

    let expected = Pixmap::load_png("tests/images/pixmap/clone-rect-2.png").unwrap();
    assert_eq!(part, expected);
}

#[test]
fn clone_rect_out_of_bound() {
    let mut canvas = Canvas::new(200, 200).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.fill_path(
        &PathBuilder::from_circle(100.0, 100.0, 80.0).unwrap(),
        &paint,
        FillRule::Winding,
    );

    assert!(canvas.pixmap.clone_rect(IntRect::from_xywh(250, 15, 80, 90).unwrap()).is_none());
    assert!(canvas.pixmap.clone_rect(IntRect::from_xywh(10, 250, 80, 90).unwrap()).is_none());
    assert!(canvas.pixmap.clone_rect(IntRect::from_xywh(10, -250, 80, 90).unwrap()).is_none());
}

#[test]
fn fill() {
    let c = Color::from_rgba8(50, 100, 150, 200);
    let mut pixmap = Pixmap::new(10, 10).unwrap();
    pixmap.fill(c);
    assert_eq!(pixmap.pixel(1, 1).unwrap(), c.premultiply().to_color_u8());
}

#[test]
fn unowned_pixmap() {
    let c = Color::from_rgba8(50, 100, 150, 200);
    let mut data = vec![0; 10*10*4];
    {
        // Create a pixmap and fill with color:
        let mut pixmap = Pixmap::from_data(10, 10, data.as_mut_slice()).unwrap();
        pixmap.fill(c);
    }

    // Create another pixmap, backed by the same data, and verify it has the right color:
    let pixmap = Pixmap::from_data(10, 10, data.as_mut_slice()).unwrap();
    assert_eq!(pixmap.pixel(1, 1).unwrap(), c.premultiply().to_color_u8())
}
