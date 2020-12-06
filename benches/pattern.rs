use bencher::{benchmark_group, benchmark_main, Bencher};

fn pattern_tiny_skia(
    quality: tiny_skia::FilterQuality,
    ts: tiny_skia::Transform,
    bencher: &mut Bencher,
) {
    use tiny_skia::*;

    fn crate_triangle() -> Pixmap {
        let mut pixmap = Pixmap::new(20, 20).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

        let mut paint = Paint::default();
        paint.set_color_rgba8(50, 127, 150, 200);
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.move_to(0.0, 20.0);
        pb.line_to(20.0, 20.0);
        pb.line_to(10.0, 0.0);
        pb.close();
        let path = pb.finish().unwrap();

        canvas.fill_path(&path, &paint, FillRule::Winding);

        pixmap
    }

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());
    let triangle = crate_triangle();

    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.shader = Pattern::new(
        triangle.as_ref(),
        SpreadMode::Repeat,
        quality,
        1.0,
        ts,
    );

    let mut pb = PathBuilder::new();
    pb.move_to(60.0, 60.0);
    pb.line_to(160.0, 940.0);
    pb.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
    pb.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
    pb.close();
    let path = pb.finish().unwrap();

    bencher.iter(|| {
        canvas.fill_path(&path, &paint, FillRule::Winding);
    });
}

fn plain_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;
    pattern_tiny_skia(
        FilterQuality::Nearest,
        Transform::identity(),
        bencher,
    )
}

fn lq_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;
    pattern_tiny_skia(
        FilterQuality::Bilinear,
        Transform::from_row(1.5, -0.4, 0.0, -0.8, 5.0, 1.0).unwrap(),
        bencher,
    )
}

fn hq_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;
    pattern_tiny_skia(
        FilterQuality::Bicubic,
        Transform::from_row(1.5, -0.4, 0.0, -0.8, 5.0, 1.0).unwrap(),
        bencher,
    )
}

fn pattern_skia(
    quality: skia_rs::FilterQuality,
    ts: skia_rs::Transform,
    bencher: &mut Bencher,
) {
    use skia_rs::*;

    fn crate_triangle() -> Surface {
        let mut surface = Surface::new_rgba_premultiplied(20, 20).unwrap();

        let mut paint = Paint::new();
        paint.set_style(PaintStyle::Fill);
        paint.set_anti_alias(true);
        paint.set_color(50, 127, 150, 200);

        let mut path = Path::new();
        path.move_to(0.0, 20.0);
        path.line_to(20.0, 20.0);
        path.line_to(10.0, 0.0);
        path.close();
        surface.draw_path(&path, &paint);

        surface
    }

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let triangle = crate_triangle();
    let shader = Shader::new_from_surface_image(
        &triangle,
        ts,
        quality,
    ).unwrap();

    let mut paint = Paint::new();
    paint.set_style(PaintStyle::Fill);
    paint.set_anti_alias(true);
    paint.set_shader(&shader);

    let mut path = Path::new();
    path.move_to(60.0, 60.0);
    path.line_to(160.0, 940.0);
    path.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
    path.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
    path.close();

    bencher.iter(|| {
        surface.draw_path(&path, &paint);
    });
}

fn plain_skia(bencher: &mut Bencher) {
    use skia_rs::*;
    pattern_skia(
        FilterQuality::None,
        Transform::default(),
        bencher,
    )
}

fn lq_skia(bencher: &mut Bencher) {
    use skia_rs::*;
    pattern_skia(
        FilterQuality::Low,
        Transform::new(1.5, -0.4, 0.0, -0.8, 5.0, 1.0),
        bencher,
    )
}

fn hq_skia(bencher: &mut Bencher) {
    use skia_rs::*;
    pattern_skia(
        FilterQuality::High,
        Transform::new(1.5, -0.4, 0.0, -0.8, 5.0, 1.0),
        bencher,
    )
}

fn pattern_cairo(
    quality: cairo::Filter,
    ts: cairo::Matrix,
    bencher: &mut Bencher,
) {
    use cairo::*;

    fn crate_triangle() -> ImageSurface {
        let surface = ImageSurface::create(Format::ARgb32, 20, 20).unwrap();

        let cr = cairo::Context::new(&surface);
        cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
        cr.set_antialias(cairo::Antialias::Subpixel);

        cr.move_to(0.0, 20.0);
        cr.line_to(20.0, 20.0);
        cr.line_to(10.0, 0.0);
        cr.close_path();

        cr.fill();

        surface
    }

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let triangle = crate_triangle();

    let cr = cairo::Context::new(&surface);

    let patt = cairo::SurfacePattern::create(&triangle);
    patt.set_extend(cairo::Extend::Repeat);
    patt.set_filter(quality);

    let mut m = ts.clone();
    m.invert();
    patt.set_matrix(m);

    cr.set_source(&patt);
    cr.set_antialias(cairo::Antialias::Subpixel);

    cr.move_to(60.0, 60.0);
    cr.line_to(160.0, 940.0);
    cr.curve_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
    cr.curve_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
    cr.close_path();

    bencher.iter(|| {
        cr.fill_preserve();
    });
}

fn plain_cairo(bencher: &mut Bencher) {
    use cairo::*;
    pattern_cairo(
        Filter::Nearest,
        Matrix::default(),
        bencher,
    )
}

fn lq_cairo(bencher: &mut Bencher) {
    use cairo::*;
    pattern_cairo(
        Filter::Bilinear,
        Matrix::new(1.5, -0.4, 0.0, -0.8, 5.0, 1.0),
        bencher,
    )
}

fn hq_cairo(bencher: &mut Bencher) {
    use cairo::*;
    pattern_cairo(
        // Looks like in cairo, the best filter is Gaussian, while in Skia it's Bicubic.
        Filter::Best,
        Matrix::new(1.5, -0.4, 0.0, -0.8, 5.0, 1.0),
        bencher,
    )
}

fn pattern_raqote(
    quality: raqote::FilterMode,
    ts: raqote::Transform,
    bencher: &mut Bencher,
) {
    use raqote::*;

    fn crate_triangle() -> DrawTarget {
        let mut dt = DrawTarget::new(20, 20);

        let mut pb = PathBuilder::new();
        pb.move_to(0.0, 20.0);
        pb.line_to(20.0, 20.0);
        pb.line_to(10.0, 00.0);
        pb.close();
        let path = pb.finish();

        // raqote uses ARGB order.
        let src = Source::from(Color::new(200, 50, 127, 150));

        let draw_opt = DrawOptions {
            blend_mode: BlendMode::SrcOver,
            alpha: 1.0,
            antialias: AntialiasMode::Gray,
        };

        dt.fill(&path, &src, &draw_opt);

        dt
    }

    let mut dt = DrawTarget::new(1000, 1000);

    let triangle = crate_triangle();

    let mut pb = PathBuilder::new();
    pb.move_to(60.0, 60.0);
    pb.line_to(160.0, 940.0);
    pb.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
    pb.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
    pb.close();
    let path = pb.finish();

    let image = raqote::Image {
        width: triangle.width() as i32,
        height: triangle.height() as i32,
        data: triangle.get_data(),
    };

    let src = raqote::Source::Image(
        image,
        ExtendMode::Repeat,
        quality,
        ts.inverse().unwrap(),
    );

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::Gray,
    };

    bencher.iter(|| {
        dt.fill(&path, &src, &draw_opt);
    });
}

fn plain_raqote(bencher: &mut Bencher) {
    use raqote::*;
    pattern_raqote(
        FilterMode::Nearest,
        Transform::default(),
        bencher,
    )
}

fn lq_raqote(bencher: &mut Bencher) {
    use raqote::*;
    pattern_raqote(
        FilterMode::Bilinear,
        Transform::row_major(1.5, -0.4, 0.0, -0.8, 5.0, 1.0),
        bencher,
    )
}

fn hq_raqote(_bencher: &mut Bencher) {
    // unsupported
}

benchmark_group!(bench,
    plain_tiny_skia,
    lq_tiny_skia,
    hq_tiny_skia,

    plain_skia,
    lq_skia,
    hq_skia,

    plain_raqote,
    lq_raqote,
    hq_raqote,

    plain_cairo,
    lq_cairo,
    hq_cairo
);
benchmark_main!(bench);
