use tiny_skia::*;

#[test]
fn empty() {
    let pb = PathBuilder::new();
    assert!(pb.finish().is_none());
}

#[test]
fn line() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.line_to(30.0, 40.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(10.0, 20.0, 30.0, 40.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(10.0, 20.0)),
        PathSegment::Line(Point::from_xy(30.0, 40.0)),
    ]);

    assert_eq!(format!("{:?}", path),
               "Path { segments: \"M 10 20 L 30 40\", \
                bounds: Bounds { left: 10, top: 20, right: 30, bottom: 40 } }");
}

#[test]
fn no_move_before_line() {
    let mut pb = PathBuilder::new();
    pb.line_to(30.0, 40.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(0.0, 0.0, 30.0, 40.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
       PathSegment::Move(Point::from_xy(0.0, 0.0)),
       PathSegment::Line(Point::from_xy(30.0, 40.0)),
    ]);
}

#[test]
fn no_move_before_quad() {
    let mut pb = PathBuilder::new();
    pb.quad_to(40.0, 30.0, 60.0, 75.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(0.0, 0.0, 60.0, 75.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
       PathSegment::Move(Point::from_xy(0.0, 0.0)),
       PathSegment::Quad(Point::from_xy(40.0, 30.0), Point::from_xy(60.0, 75.0)),
    ]);
}

#[test]
fn no_move_before_cubic() {
    let mut pb = PathBuilder::new();
    pb.cubic_to(40.0, 30.0, 60.0, 75.0, 33.0, 66.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(0.0, 0.0, 60.0, 75.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(0.0, 0.0)),
        PathSegment::Cubic(Point::from_xy(40.0, 30.0), Point::from_xy(60.0, 75.0), Point::from_xy(33.0, 66.0)),
    ]);
}

#[test]
fn no_move_before_close() {
    let mut pb = PathBuilder::new();
    pb.close();
    assert!(pb.finish().is_none());
}

#[test]
fn double_close() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(20.0, 10.0);
    pb.line_to(20.0, 20.0);
    pb.close();
    pb.close();
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(10.0, 10.0, 20.0, 20.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(10.0, 10.0)),
        PathSegment::Line(Point::from_xy(20.0, 10.0)),
        PathSegment::Line(Point::from_xy(20.0, 20.0)),
        PathSegment::Close,
    ]);
}

#[test]
fn double_move_to_1() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.move_to(30.0, 40.0);
    assert!(pb.finish().is_none());
}

#[test]
fn double_move_to_2() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.move_to(20.0, 10.0);
    pb.line_to(30.0, 40.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(20.0, 10.0, 30.0, 40.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(20.0, 10.0)),
        PathSegment::Line(Point::from_xy(30.0, 40.0)),
    ]);
}

#[test]
fn two_contours() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.line_to(30.0, 40.0);
    pb.move_to(100.0, 200.0);
    pb.line_to(300.0, 400.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(10.0, 20.0, 300.0, 400.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(10.0, 20.0)),
        PathSegment::Line(Point::from_xy(30.0, 40.0)),
        PathSegment::Move(Point::from_xy(100.0, 200.0)),
        PathSegment::Line(Point::from_xy(300.0, 400.0)),
    ]);
}

#[test]
fn two_closed_contours() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.line_to(30.0, 40.0);
    pb.close();
    pb.move_to(100.0, 200.0);
    pb.line_to(300.0, 400.0);
    pb.close();
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(10.0, 20.0, 300.0, 400.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(10.0, 20.0)),
        PathSegment::Line(Point::from_xy(30.0, 40.0)),
        PathSegment::Close,
        PathSegment::Move(Point::from_xy(100.0, 200.0)),
        PathSegment::Line(Point::from_xy(300.0, 400.0)),
        PathSegment::Close,
    ]);
}

#[test]
fn line_after_close() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 20.0);
    pb.line_to(30.0, 40.0);
    pb.close();
    pb.line_to(20.0, 20.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(10.0, 20.0, 30.0, 40.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(10.0, 20.0)),
        PathSegment::Line(Point::from_xy(30.0, 40.0)),
        PathSegment::Close,
        PathSegment::Move(Point::from_xy(10.0, 20.0)),
        PathSegment::Line(Point::from_xy(20.0, 20.0)),
    ]);
}

#[test]
fn hor_line() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(20.0, 10.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(10.0, 10.0, 20.0, 10.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(10.0, 10.0)),
        PathSegment::Line(Point::from_xy(20.0, 10.0)),
    ]);
}

#[test]
fn ver_line() {
    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.line_to(10.0, 20.0);
    let path = pb.finish().unwrap();

    assert_eq!(path.bounds(), Bounds::from_ltrb(10.0, 10.0, 10.0, 20.0).unwrap());
    assert_eq!(path.segments().collect::<Vec<_>>(), &[
        PathSegment::Move(Point::from_xy(10.0, 10.0)),
        PathSegment::Line(Point::from_xy(10.0, 20.0)),
    ]);
}
