use tiny_skia::*;

#[test]
fn horizontal_line() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(90.0, 10.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/empty.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn vertical_line() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(10.0, 90.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/empty.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn single_line() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(90.0, 90.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/empty.png").unwrap();

    assert_eq!(pixmap, expected);
}


#[test]
fn open_polygon() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(75.160671, 88.756136);
    pb.line_to(24.797274, 88.734053);
    pb.line_to( 9.255130, 40.828792);
    pb.line_to(50.012955, 11.243795);
    pb.line_to(90.744819, 40.864522);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/polygon.png").unwrap();

    assert_eq!(pixmap, expected);
}

// Must be the same a open.
#[test]
fn closed_polygon() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(75.160671, 88.756136);
    pb.line_to(24.797274, 88.734053);
    pb.line_to( 9.255130, 40.828792);
    pb.line_to(50.012955, 11.243795);
    pb.line_to(90.744819, 40.864522);
    pb.close(); // the only difference
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/polygon.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn winding_star() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        fill_type: FillType::Winding,
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(50.0,  7.5);
    pb.line_to(75.0, 87.5);
    pb.line_to(10.0, 37.5);
    pb.line_to(90.0, 37.5);
    pb.line_to(25.0, 87.5);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/winding-star.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn even_odd_star() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        fill_type: FillType::EvenOdd,
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(50.0,  7.5);
    pb.line_to(75.0, 87.5);
    pb.line_to(10.0, 37.5);
    pb.line_to(90.0, 37.5);
    pb.line_to(25.0, 87.5);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/even-odd-star.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn quad_curve() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        fill_type: FillType::EvenOdd,
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 15.0);
    pb.quad_to(95.0, 35.0, 75.0, 90.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/quad.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn cubic_curve() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        fill_type: FillType::EvenOdd,
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 15.0);
    pb.cubic_to(95.0, 35.0, 0.0, 75.0, 75.0, 90.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/cubic.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn memset2d() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 255), // Must be opaque to trigger memset2d.
        ..Paint::default()
    };

    let path = PathBuilder::from_bound(Bounds::from_ltrb(10.0, 10.0, 90.0, 90.0).unwrap());
    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/memset2d.png").unwrap();

    assert_eq!(pixmap, expected);
}

// Make sure we do not write past pixmap memory.
#[test]
fn memset2d_out_of_bounds() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 255), // Must be opaque to trigger memset2d.
        ..Paint::default()
    };

    let path = PathBuilder::from_bound(Bounds::from_ltrb(50.0, 50.0, 120.0, 120.0).unwrap());
    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/memset2d-2.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn fill_pixmap() {
    let mut pixmap = Pixmap::new(10, 10).unwrap();
    let c = Color::from_rgba8(50, 100, 150, 200);
    pixmap.fill(c);
    assert_eq!(pixmap.pixel(1, 1).unwrap(), c.premultiply().to_color_u8());
}

// Not sure how to properly test anti-aliasing,
// so for now simply check that it actually applied.
#[test]
fn fill_aa() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        fill_type: FillType::EvenOdd,
        anti_alias: true,
        ..Paint::default()
    };

    let mut pb = PathBuilder::new();
    pb.move_to(50.0,  7.5);
    pb.line_to(75.0, 87.5);
    pb.line_to(10.0, 37.5);
    pb.line_to(90.0, 37.5);
    pb.line_to(25.0, 87.5);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/star-aa.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn overflow_in_walk_edges_1() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        blend_mode: BlendMode::default(),
        fill_type: FillType::Winding,
        anti_alias: false,
    };

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.cubic_to(39.0, 163.0, 117.0, 61.0, 130.0, 70.0);
    let path = pb.finish().unwrap();

    // Must not panic.
    pixmap.fill_path(&path, &paint);
}

#[test]
fn clip_line_1() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        blend_mode: BlendMode::default(),
        fill_type: FillType::Winding,
        anti_alias: false,
    };

    let mut pb = PathBuilder::new();
    pb.move_to(50.0, -15.0);
    pb.line_to(-15.0, 50.0);
    pb.line_to(50.0, 115.0);
    pb.line_to(115.0, 50.0);
    pb.close();
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/clip-line-1.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn clip_line_2() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        blend_mode: BlendMode::default(),
        fill_type: FillType::Winding,
        anti_alias: false,
    };

    // This strange path forces `line_clipper::clip` to return an empty array.
    // And we're checking that this case is handled correctly.
    let mut pb = PathBuilder::new();
    pb.move_to(0.0, -1.0);
    pb.line_to(50.0, 0.0);
    pb.line_to(0.0, 50.0);
    pb.close();
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/clip-line-2.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn clip_quad() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        blend_mode: BlendMode::default(),
        fill_type: FillType::Winding,
        anti_alias: false,
    };

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 85.0);
    pb.quad_to(150.0, 150.0, 85.0, 15.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/clip-quad.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn clip_cubic_1() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        blend_mode: BlendMode::default(),
        fill_type: FillType::Winding,
        anti_alias: false,
    };

    // `line_clipper::clip` produces 2 points for this path.
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 50.0);
    pb.cubic_to(0.0, 175.0, 195.0, 70.0, 75.0, 20.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/clip-cubic-1.png").unwrap();

    assert_eq!(pixmap, expected);
}

#[test]
fn clip_cubic_2() {
    let mut pixmap = Pixmap::new(100, 100).unwrap();

    let paint = Paint {
        color: Color::from_rgba8(50, 127, 150, 200),
        blend_mode: BlendMode::default(),
        fill_type: FillType::Winding,
        anti_alias: false,
    };

    // `line_clipper::clip` produces 3 points for this path.
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 50.0);
    pb.cubic_to(10.0, 40.0, 90.0, 120.0, 125.0, 20.0);
    let path = pb.finish().unwrap();

    pixmap.fill_path(&path, &paint);
    let expected = Pixmap::load_png("tests/images/fill/clip-cubic-2.png").unwrap();

    assert_eq!(pixmap, expected);
}
