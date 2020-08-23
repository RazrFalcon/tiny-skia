use tiny_skia::*;

#[test]
fn zero_len_subpath_butt_cap() {
    let mut pb = PathBuilder::new();
    pb.move_to(100.0, 100.0);
    pb.line_to(100.0, 100.0);
    let path = pb.finish().unwrap();

    let props = StrokeProps::default()
        .set_width(20.0)
        .set_line_cap(LineCap::Butt);

    // A zero-len subpath with a butt line cap produces nothing.
    assert_eq!(path.stroke(props), None);
}

#[test]
fn zero_len_subpath_round_cap() {
    let mut pb = PathBuilder::new();
    pb.move_to(100.0, 100.0);
    pb.line_to(100.0, 100.0);
    let path = pb.finish().unwrap();

    let props = StrokeProps::default()
        .set_width(20.0)
        .set_line_cap(LineCap::Round);

    // A zero-len subpath with a round line cap produces a circle.
    let stroke_path = path.stroke(props).unwrap();

    // Skia sure spams a lot...
    let expected = {
        let mut pb = PathBuilder::new();
        pb.move_to(110.0, 100.0);
        pb.line_to(110.0, 100.0);
        pb.quad_to(109.99999, 100.98491, 109.80784, 101.95089);
        pb.quad_to(109.61569, 102.91688, 109.23878, 103.82682);
        pb.quad_to(108.86187, 104.736755, 108.31468, 105.55569);
        pb.quad_to(107.767494, 106.37462, 107.07106, 107.07106);
        pb.quad_to(106.37462, 107.767494, 105.55568, 108.31468);
        pb.quad_to(104.736755, 108.86187, 103.82682, 109.238785);
        pb.quad_to(102.91688, 109.61569, 101.9509, 109.807846);
        pb.quad_to(100.98491, 109.99999, 100.0, 110.0);
        pb.quad_to(99.01508, 109.99999, 98.049095, 109.80784);
        pb.quad_to(97.0831, 109.61569, 96.17316, 109.23878);
        pb.quad_to(95.26322, 108.86187, 94.44429, 108.31468);
        pb.quad_to(93.62537, 107.767494, 92.92893, 107.07106);
        pb.quad_to(92.23248, 106.37462, 91.685295, 105.55568);
        pb.quad_to(91.13811, 104.736755, 90.7612, 103.82682);
        pb.quad_to(90.38429, 102.91688, 90.19215, 101.9509);
        pb.quad_to(90.0, 100.98491, 90.0, 100.0);
        pb.line_to(90.0, 100.0);
        pb.quad_to(90.0, 99.01508, 90.19215, 98.049095);
        pb.quad_to(90.38429, 97.0831, 90.7612, 96.17316);
        pb.quad_to(91.13811, 95.26322, 91.685295, 94.44429);
        pb.quad_to(92.23248, 93.62537, 92.92893, 92.92893);
        pb.quad_to(93.62537, 92.23248, 94.44429, 91.685295);
        pb.quad_to(95.26322, 91.13811, 96.17316, 90.7612);
        pb.quad_to(97.0831, 90.38429, 98.049095, 90.19215);
        pb.quad_to(99.01508, 90.0, 100.0, 90.0);
        pb.quad_to(100.98491, 90.0, 101.95089, 90.19215);
        pb.quad_to(102.91688, 90.38429, 103.82682, 90.7612);
        pb.quad_to(104.736755, 91.13811, 105.55569, 91.685295);
        pb.quad_to(106.37462, 92.23248, 107.07106, 92.92893);
        pb.quad_to(107.767494, 93.62537, 108.31468, 94.44429);
        pb.quad_to(108.86187, 95.26322, 109.238785, 96.17316);
        pb.quad_to(109.61569, 97.0831, 109.807846, 98.049095);
        pb.quad_to(109.99999, 99.01508, 110.0, 100.0);
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

    let props = StrokeProps::default()
        .set_width(20.0)
        .set_line_cap(LineCap::Square);

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
