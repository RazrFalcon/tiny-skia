use tiny_skia::*;

#[test]
fn zero_len_subpath_butt_cap() {
    let mut pb = PathBuilder::new();
    pb.move_to(100.0, 100.0);
    pb.line_to(100.0, 100.0);
    let path = pb.finish().unwrap();

    let mut props = StrokeProps::default();
    props.width = 20.0;
    props.line_cap = LineCap::Butt;

    // A zero-len subpath with a butt line cap produces nothing.
    assert_eq!(path.stroke(props), None);
}

#[test]
fn zero_len_subpath_round_cap() {
    let mut pb = PathBuilder::new();
    pb.move_to(100.0, 100.0);
    pb.line_to(100.0, 100.0);
    let path = pb.finish().unwrap();

    let mut props = StrokeProps::default();
    props.width = 20.0;
    props.line_cap = LineCap::Round;

    // A zero-len subpath with a round line cap produces a circle.
    let stroke_path = path.stroke(props).unwrap();

    // Skia sure spams a lot...
    let expected = {
        let mut pb = PathBuilder::new();
        pb.move_to(110.0, 100.0);
        pb.line_to(110.0, 100.0);
        pb.quad_to(109.99999, 104.14213, 107.07106, 107.07106);
        pb.quad_to(104.14213, 109.99999, 100.0, 110.0);
        pb.quad_to(95.857864, 109.99999, 92.92893, 107.07106);
        pb.quad_to(90.0, 104.14213, 90.0, 100.0);
        pb.line_to(90.0, 100.0);
        pb.quad_to(90.0, 95.857864, 92.92893, 92.92893);
        pb.quad_to(95.857864, 90.0, 100.0, 90.0);
        pb.quad_to(104.14213, 90.0, 107.07106, 92.92893);
        pb.quad_to(109.99999, 95.857864, 110.0, 100.0);
        pb.close();
        pb.finish().unwrap()
    };

    assert_eq!(stroke_path, expected);
}

#[test]
fn zero_len_subpath_square_cap() {
    let mut pb = PathBuilder::new();
    pb.move_to(100.0, 100.0);
    pb.line_to(100.0, 100.0);
    let path = pb.finish().unwrap();

    let mut props = StrokeProps::default();
    props.width = 20.0;
    props.line_cap = LineCap::Square;

    // A zero-len subpath with a round line cap produces a circle.
    let stroke_path = path.stroke(props).unwrap();

    let expected = {
        let mut pb = PathBuilder::new();
        pb.move_to(110.0, 100.0);
        pb.line_to(110.0, 110.0);
        pb.line_to(90.0, 110.0);
        pb.line_to(90.0, 90.0);
        pb.line_to(110.0, 90.0);
        pb.close();
        pb.finish().unwrap()
    };

    assert_eq!(stroke_path, expected);
}

// Make sure that subpath auto-closing is enabled.
#[test]
fn auto_close() {
    // A triangle.
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(20.0, 50.0);
    pb.line_to(30.0, 10.0);
    pb.close();
    let path = pb.finish().unwrap();

    let props = StrokeProps::default();
    let stroke_path = path.stroke(props).unwrap();

    let mut iter = stroke_path.segments();
    iter.set_auto_close(true);

    assert_eq!(iter.next().unwrap(), PathSegment::new_move_to(10.485071, 9.878732));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(20.485071, 49.878731));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(20.0, 50.0));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(19.514929, 49.878731));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(29.514929, 9.878732));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(30.0, 10.0));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(30.0, 10.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(10.0, 10.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(10.0, 10.0));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(10.485071, 9.878732));
    assert_eq!(iter.next().unwrap(), PathSegment::new_close());
    assert_eq!(iter.next().unwrap(), PathSegment::new_move_to(9.3596115, 9.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(30.640388, 9.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(20.485071, 50.121269));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(19.514929, 50.121269));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(9.514929, 10.121268));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(9.3596115, 9.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_close());
}
