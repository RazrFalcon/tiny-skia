use bencher::{benchmark_group, benchmark_main, Bencher};

// Filling a semi-transparent rectangle path with a Source blending mode.
// By using this blending mode we're forcing a simple pixels overwrite.
fn source_fill_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut canvas = Canvas::new(1000, 1000).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.blend_mode = BlendMode::Source;

    let path = PathBuilder::from_bounds(Bounds::from_ltrb(100.0, 100.0, 900.0, 900.0).unwrap());

    bencher.iter(|| {
        canvas.fill_path(&path, &paint, FillType::Winding);
    });
}

fn source_fill_skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);
    paint.set_blend_mode(BlendMode::Source);

    let mut path = Path::new();
    path.push_rect(100.0, 100.0, 900.0, 900.0);

    bencher.iter(|| {
        surface.draw_path(&path, &paint);
    });
}

fn source_fill_raqote(bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    let path = {
        let mut pb = PathBuilder::new();
        pb.rect(100.0, 100.0, 800.0, 800.0);
        pb.finish()
    };

    // raqote uses ARGB order.
    let src = Source::from(Color::new(200, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::Src,
        alpha: 1.0,
        antialias: AntialiasMode::None,
    };

    bencher.iter(|| {
        dt.fill(&path, &src, &draw_opt);
    });
}

fn source_fill_cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);

    cr.rectangle(100.0, 100.0, 800.0, 800.0);

    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
    cr.set_antialias(Antialias::None);
    cr.set_fill_rule(FillRule::Winding);
    cr.set_operator(Operator::Source);

    bencher.iter(|| {
        cr.fill_preserve();
    });
}


// Filling an opaque rectangle path.
// A chosen blending mode doesn't really matter in this case,
// since we are simply overwriting the pixels.
fn opaque_fill_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut canvas = Canvas::new(1000, 1000).unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 255);

    let path = PathBuilder::from_bounds(Bounds::from_ltrb(100.0, 100.0, 900.0, 900.0).unwrap());

    bencher.iter(|| {
        canvas.fill_path(&path, &paint, FillType::Winding);
    });
}

fn opaque_fill_skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 255);
    paint.set_blend_mode(BlendMode::SourceOver);

    let mut path = Path::new();
    path.push_rect(100.0, 100.0, 900.0, 900.0);

    bencher.iter(|| {
        surface.draw_path(&path, &paint);
    });
}

fn opaque_fill_raqote(bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    let path = {
        let mut pb = PathBuilder::new();
        pb.rect(100.0, 100.0, 800.0, 800.0);
        pb.finish()
    };

    // raqote uses ARGB order.
    let src = Source::from(Color::new(255, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::None,
    };

    bencher.iter(|| {
        dt.fill(&path, &src, &draw_opt);
    });
}

fn opaque_fill_cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);

    cr.rectangle(100.0, 100.0, 800.0, 800.0);

    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 255.0 / 255.0);
    cr.set_antialias(Antialias::None);
    cr.set_fill_rule(FillRule::Winding);
    cr.set_operator(Operator::Over);

    bencher.iter(|| {
        cr.fill_preserve();
    });
}

benchmark_group!(fill,
    source_fill_tiny_skia,
    source_fill_skia,
    source_fill_raqote,
    source_fill_cairo,

    opaque_fill_tiny_skia,
    opaque_fill_skia,
    opaque_fill_raqote,
    opaque_fill_cairo
);
benchmark_main!(fill);
