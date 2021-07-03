use test::Bencher;

#[bench]
fn tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let rect = Rect::from_xywh(50.7, 20.1, 812.4, 777.3).unwrap();

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();

    bencher.iter(|| {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    });
}

#[cfg(feature = "skia-rs")]
#[bench]
fn skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);

    bencher.iter(|| {
        surface.draw_rect(50.7, 20.1, 812.4, 777.3, &paint)
    });
}

#[cfg(feature = "raqote")]
#[bench]
fn raqote(bencher: &mut Bencher) {
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

#[cfg(feature = "cairo-rs")]
#[bench]
fn cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);
    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
    cr.rectangle(50.7, 20.1, 812.4, 777.3);

    bencher.iter(|| {
        cr.fill_preserve();
    });
}

#[bench]
fn aa_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(50.7, 20.1, 812.4, 777.3).unwrap();

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();

    bencher.iter(|| {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    });
}

#[cfg(feature = "skia-rs")]
#[bench]
fn aa_skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);
    paint.set_anti_alias(true);

    bencher.iter(|| {
        surface.draw_rect(50.7, 20.1, 812.4, 777.3, &paint)
    });
}

#[cfg(feature = "raqote")]
#[bench]
fn aa_raqote(bencher: &mut Bencher) {
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

#[cfg(feature = "cairo-rs")]
#[bench]
fn aa_cairo(bencher: &mut Bencher) {
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

#[bench]
fn aa_ts_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let rect = Rect::from_xywh(200.3, 100.4, 500.5, 300.2).unwrap();

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();

    bencher.iter(|| {

        pixmap.fill_rect(rect, &paint, Transform::from_row(1.8, 0.3, -0.7, 0.8, 12.0, 15.3), None);
    });
}

#[cfg(feature = "skia-rs")]
#[bench]
fn aa_ts_skia(bencher: &mut Bencher) {
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

#[cfg(feature = "raqote")]
#[bench]
fn aa_ts_raqote(bencher: &mut Bencher) {
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

#[cfg(feature = "cairo-rs")]
#[bench]
fn aa_ts_cairo(bencher: &mut Bencher) {
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
