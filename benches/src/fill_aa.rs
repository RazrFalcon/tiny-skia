use test::Bencher;

#[bench]
fn tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let mut pb = PathBuilder::new();
    pb.move_to(500.0, 20.0);
    pb.cubic_to(650.0, 320.0, 770.0, 650.0, 800.0, 980.0);
    pb.line_to(20.0, 380.0);
    pb.line_to(200.0, 980.0);
    pb.cubic_to(230.0, 650.0, 350.0, 320.0, 500.0, 20.0);
    pb.close();
    let path = pb.finish().unwrap();

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();

    bencher.iter(|| {
        pixmap.fill_path(&path, &paint, FillRule::EvenOdd, Transform::identity(), None);
    });
}

#[cfg(feature = "skia-rs")]
#[bench]
fn skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);
    paint.set_blend_mode(BlendMode::SourceOver);
    paint.set_anti_alias(true);

    let mut path = Path::new();
    path.move_to(500.0, 20.0);
    path.cubic_to(650.0, 320.0, 770.0, 650.0, 800.0, 980.0);
    path.line_to(20.0, 380.0);
    path.line_to(200.0, 980.0);
    path.cubic_to(230.0, 650.0, 350.0, 320.0, 500.0, 20.0);
    path.close();
    path.set_fill_type(FillType::EvenOdd);

    bencher.iter(|| {
        surface.draw_path(&path, &paint);
    });
}

#[cfg(feature = "raqote")]
#[bench]
fn raqote(bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    let mut path = {
        let mut pb = PathBuilder::new();
        pb.move_to(500.0, 20.0);
        pb.cubic_to(650.0, 320.0, 770.0, 650.0, 800.0, 980.0);
        pb.line_to(20.0, 380.0);
        pb.line_to(200.0, 980.0);
        pb.cubic_to(230.0, 650.0, 350.0, 320.0, 500.0, 20.0);
        pb.close();
        pb.finish()
    };
    path.winding = Winding::EvenOdd;

    // raqote uses ARGB order.
    let src = Source::from(Color::new(200, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::Gray,
    };

    bencher.iter(|| {
        dt.fill(&path, &src, &draw_opt);
    });
}

#[cfg(feature = "cairo-rs")]
#[bench]
fn cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);

    cr.move_to(500.0, 20.0);
    cr.curve_to(650.0, 320.0, 770.0, 650.0, 800.0, 980.0);
    cr.line_to(20.0, 380.0);
    cr.line_to(200.0, 980.0);
    cr.curve_to(230.0, 650.0, 350.0, 320.0, 500.0, 20.0);
    cr.close_path();

    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
    cr.set_antialias(Antialias::Subpixel); // TODO: or Gray?
    cr.set_fill_rule(FillRule::EvenOdd);
    cr.set_operator(Operator::Over);

    bencher.iter(|| {
        cr.fill_preserve();
    });
}
