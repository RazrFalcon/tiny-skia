use bencher::{benchmark_group, benchmark_main, Bencher};

fn fill_aa_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut canvas = Canvas::new(1000, 1000).unwrap();

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

    bencher.iter(|| {
        canvas.fill_path(&path, &paint, FillRule::EvenOdd);
    });
}

fn fill_aa_skia(bencher: &mut Bencher) {
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

fn fill_aa_raqote(bencher: &mut Bencher) {
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

fn fill_aa_cairo(bencher: &mut Bencher) {
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

benchmark_group!(fill_aa,
    fill_aa_tiny_skia,
    fill_aa_skia,
    fill_aa_raqote,
    fill_aa_cairo
);
benchmark_main!(fill_aa);
