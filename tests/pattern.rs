use tiny_skia::*;

fn crate_triangle() -> Pixmap {
    let mut pixmap = Pixmap::new(20, 20).unwrap();

    let paint = Paint::default()
        .set_color_rgba8(50, 127, 150, 200)
        .set_anti_alias(true);

    let mut pb = PathBuilder::new();
    pb.move_to(0.0, 20.0);
    pb.line_to(20.0, 20.0);
    pb.line_to(10.0, 0.0);
    pb.close();
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);

    pixmap
}

#[test]
fn filter_nearest_neighbor_no_ts() {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let paint = Paint::default()
        .set_shader(Pattern::new(
            &triangle,
            FilterQuality::Nearest,
            Transform::identity(),
        ));

    let path = PathBuilder::from_bound(Bounds::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    pixmap.fill_path(&path, &paint);

    let expected = Pixmap::load_png("tests/images/pattern/filter-nearest-neighbor-no-ts.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn filter_nearest_neighbor() {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let paint = Paint::default()
        .set_shader(Pattern::new(
            &triangle,
            FilterQuality::Nearest,
            Transform::from_row(1.5, -0.4, 0.0, -0.8, 5.0, 1.0).unwrap(),
        ));

    let path = PathBuilder::from_bound(Bounds::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    pixmap.fill_path(&path, &paint);

    let expected = Pixmap::load_png("tests/images/pattern/filter-nearest-neighbor.png").unwrap();
    assert_eq!(pixmap, expected);
}

#[test]
fn filter_bilinear() {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let paint = Paint::default()
        .set_shader(Pattern::new(
            &triangle,
            FilterQuality::Bilinear,
            Transform::from_row(1.5, -0.4, 0.0, -0.8, 5.0, 1.0).unwrap(),
        ));

    let path = PathBuilder::from_bound(Bounds::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    pixmap.fill_path(&path, &paint);

    // SIMD and non-SIMD version produce a slightly different results. Not sure why.
    #[cfg(all(feature = "sse2", target_feature = "sse2"))]
    {
        let expected = Pixmap::load_png("tests/images/pattern/filter-bilinear.png").unwrap();
        assert_eq!(pixmap, expected);
    }

    #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
    {
        let expected = Pixmap::load_png("tests/images/pattern/filter-bilinear-no-simd.png").unwrap();
        assert_eq!(pixmap, expected);
    }
}

#[test]
fn filter_bicubic() {
    let mut pixmap = Pixmap::new(200, 200).unwrap();
    let triangle = crate_triangle();

    let paint = Paint::default()
        .set_shader(Pattern::new(
            &triangle,
            FilterQuality::Bicubic,
            Transform::from_row(1.5, -0.4, 0.0, -0.8, 5.0, 1.0).unwrap(),
        ));

    let path = PathBuilder::from_bound(Bounds::from_ltrb(10.0, 10.0, 190.0, 190.0).unwrap());

    pixmap.fill_path(&path, &paint);

    let expected = Pixmap::load_png("tests/images/pattern/filter-bicubic.png").unwrap();
    assert_eq!(pixmap, expected);
}
