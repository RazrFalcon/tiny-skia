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

#[test]
fn circle() {
    let path = PathBuilder::from_circle(100.0, 100.0, 50.0).unwrap();
    let props = StrokeProps::default();
    let stroke_path = path.stroke(props).unwrap();

    let mut iter = stroke_path.segments();
    assert_eq!(iter.next().unwrap(), PathSegment::new_move_to(150.5, 100.0));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(150.5, 110.04529, 146.6559, 119.3255));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(142.81177, 128.60547, 135.7089, 135.70888));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(128.60571, 142.81201, 119.32549, 146.6559));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(110.045166, 150.5, 100.0, 150.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(89.95471, 150.5, 80.674484, 146.6559));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(71.394165, 142.81177, 64.2911, 135.70888));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(57.188354, 128.6062, 53.344074, 119.3255));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(49.49994, 110.045166, 49.5, 100.0));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(49.5, 89.954834, 53.344074, 80.67448));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(57.188232, 71.39404, 64.2911, 64.2911));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(71.39392, 57.18811, 80.67448, 53.344078));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(89.954834, 49.49994, 100.0, 49.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(110.045044, 49.5, 119.32551, 53.344078));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(128.60645, 57.188354, 135.70888, 64.2911));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(142.81177, 71.39404, 146.6559, 80.674484));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(150.5, 89.954834, 150.5, 100.0));
    assert_eq!(iter.next().unwrap(), PathSegment::Close);
    assert_eq!(iter.next().unwrap(), PathSegment::new_move_to(149.5, 100.0));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(149.5, 90.15369, 145.73201, 81.05716));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(141.96411, 71.96057, 135.00179, 64.99821));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(128.0398, 58.03607, 118.94282, 54.26796));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(109.84631, 50.5, 100.0, 50.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(90.15381, 50.50006, 81.05717, 54.26796));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(71.96045, 58.03589, 64.99821, 64.99821));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(58.03595, 71.96045, 54.267956, 81.05717));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(50.5, 90.15381, 50.5, 100.0));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(50.50006, 109.84619, 54.267956, 118.94281));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(58.036133, 128.0398, 64.99821, 135.00179));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(71.96057, 141.96387, 81.05716, 145.73201));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(90.153564, 149.5, 100.0, 149.5));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(109.84619, 149.5, 118.94282, 145.73201));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(128.03906, 141.96411, 135.00177, 135.00179));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(141.96411, 128.03906, 145.73201, 118.942825));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(149.5, 109.84631, 149.5, 100.0));
    assert_eq!(iter.next().unwrap(), PathSegment::Close);
}

#[test]
fn round_cap_join() {
    let mut pb = PathBuilder::new();
    pb.move_to(170.0, 30.0);
    pb.line_to(30.553378, 99.048418);
    pb.cubic_to(30.563658, 99.066835, 30.546308, 99.280724, 30.557592, 99.305282);
    let path = pb.finish().unwrap();

    let mut props = StrokeProps::default();
    props.width = 30.0;
    props.line_cap = LineCap::Round;
    props.line_join = LineJoin::Round;
    let stroke_path = path.stroke(props).unwrap();

    let mut iter = stroke_path.segments();
    assert_eq!(iter.next().unwrap(), PathSegment::new_move_to(176.65611, 43.44233));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(37.209484, 112.490746));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(30.553377, 99.048416));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(43.650993, 91.7373));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(45.667908, 95.35053, 45.549374, 99.58929));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(45.64382, 96.21188, 44.187744, 93.04278));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(46.781757, 98.68856, 44.62382, 104.514984));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(42.465885, 110.34141, 36.820095, 112.93543));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(31.174297, 115.52944, 25.347874, 113.371506));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(19.52145, 111.21357, 16.927439, 105.56779));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(15.459791, 102.373505, 15.561099, 98.750694));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(15.448862, 102.76424, 17.455761, 106.359535));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(17.275322, 106.036285, 17.111046, 105.70452));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(14.353994, 100.13652, 16.341633, 94.24983));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(18.329273, 88.36313, 23.89727, 85.60609));
    assert_eq!(iter.next().unwrap(), PathSegment::new_line_to(163.34389, 16.55767));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(168.91187, 13.800619, 174.79857, 15.788258));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(180.68527, 17.775896, 183.44234, 23.343893));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(186.19939, 28.911888, 184.21175, 34.798584));
    assert_eq!(iter.next().unwrap(), PathSegment::new_quad_to(182.2241, 40.685276, 176.65611, 43.44233));
    assert_eq!(iter.next().unwrap(), PathSegment::Close);
}
