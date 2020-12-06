use bencher::{benchmark_group, benchmark_main, Bencher};

fn fill_rect_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let rect = Rect::from_xywh(50.7, 20.1, 812.4, 777.3).unwrap();

    bencher.iter(|| {
        canvas.fill_rect(rect, &paint);
    });
}

fn fill_rect_skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);

    bencher.iter(|| {
        surface.draw_rect(50.7, 20.1, 812.4, 777.3, &paint)
    });
}

fn fill_rect_raqote(bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    // raqote uses ARGB order.
    let src = Source::from(Color::new(200, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::None,
    };

    bencher.iter(|| {
        dt.fill_rect(50.7, 20.1, 812.4, 777.3, &src, &draw_opt);
    });
}

fn fill_rect_cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);
    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
    cr.rectangle(50.7, 20.1, 812.4, 777.3);

    bencher.iter(|| {
        cr.fill_preserve();
    });
}

fn fill_rect_aa_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(50.7, 20.1, 812.4, 777.3).unwrap();

    bencher.iter(|| {
        canvas.fill_rect(rect, &paint);
    });
}

fn fill_rect_aa_skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);
    paint.set_anti_alias(true);

    bencher.iter(|| {
        surface.draw_rect(50.7, 20.1, 812.4, 777.3, &paint)
    });
}

fn fill_rect_aa_raqote(bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    // raqote uses ARGB order.
    let src = Source::from(Color::new(200, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::Gray,
    };

    bencher.iter(|| {
        dt.fill_rect(50.7, 20.1, 812.4, 777.3, &src, &draw_opt);
    });
}

fn fill_rect_aa_cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);
    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
    cr.set_antialias(Antialias::Subpixel); // TODO: or Gray?
    cr.rectangle(50.7, 20.1, 812.4, 777.3);

    bencher.iter(|| {
        cr.fill_preserve();
    });
}

fn fill_rect_aa_ts_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();
    let mut canvas = Canvas::from(pixmap.as_mut());

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    canvas.transform(1.8, 0.3, -0.7, 0.8, 12.0, 15.3);
    let rect = Rect::from_xywh(200.3, 100.4, 500.5, 300.2).unwrap();

    bencher.iter(|| {
        canvas.fill_rect(rect, &paint);
    });
}

fn fill_rect_aa_ts_skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);
    paint.set_anti_alias(true);

    surface.concat(Transform::new(1.8, 0.3, -0.7, 0.8, 12.0, 15.3));

    bencher.iter(|| {
        surface.draw_rect(200.3, 100.4, 500.5, 300.2, &paint)
    });
}

fn fill_rect_aa_ts_raqote(bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    // raqote uses ARGB order.
    let src = Source::from(Color::new(200, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::Gray,
    };

    dt.set_transform(&Transform::row_major(1.8, 0.3, -0.7, 0.8, 12.0, 15.3));

    bencher.iter(|| {
        dt.fill_rect(200.3, 100.4, 500.5, 300.2, &src, &draw_opt);
    });
}

fn fill_rect_aa_ts_cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);
    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
    cr.set_antialias(Antialias::Subpixel); // TODO: or Gray?
    cr.transform(Matrix::new(1.8, 0.3, -0.7, 0.8, 12.0, 15.3));
    cr.rectangle(200.3, 100.4, 500.5, 300.2);

    bencher.iter(|| {
        cr.fill_preserve();
    });
}

benchmark_group!(fill_rect,
    fill_rect_tiny_skia,
    fill_rect_skia,
    fill_rect_raqote,
    fill_rect_cairo,

    fill_rect_aa_tiny_skia,
    fill_rect_aa_skia,
    fill_rect_aa_raqote,
    fill_rect_aa_cairo,

    fill_rect_aa_ts_tiny_skia,
    fill_rect_aa_ts_skia,
    fill_rect_aa_ts_raqote,
    fill_rect_aa_ts_cairo
);
benchmark_main!(fill_rect);
