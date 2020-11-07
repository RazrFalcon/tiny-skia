use bencher::{benchmark_group, benchmark_main, Bencher};

fn do_clip_tiny_skia(aa: bool, bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);

    let clip_path = {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_rect(100.0, 100.0, 800.0, 800.0);
        pb.push_rect(300.0, 300.0, 400.0, 400.0);
        pb.finish().unwrap()
    };

    let path = tiny_skia::PathBuilder::from_rect(Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0).unwrap());

    let mut canvas = Canvas::new(1000, 1000).unwrap();
    bencher.iter(|| {
        canvas.set_transform(Transform::from_row(1.0, -0.5, 0.0, 1.0, 0.0, 300.0).unwrap());
        canvas.set_clip_path(&clip_path, FillRule::EvenOdd, aa);
        canvas.reset_transform();
        // Do not use fill_rect, because it is very slow by itself.
        canvas.fill_path(&path, &paint, FillRule::Winding);
    });
}

fn clip_path_tiny_skia(bencher: &mut Bencher) {
    do_clip_tiny_skia(false, bencher);
}

fn clip_path_aa_tiny_skia(bencher: &mut Bencher) {
    do_clip_tiny_skia(true, bencher);
}

fn do_clip_path_skia(aa: bool, bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint = Paint::new();
    paint.set_color(50, 127, 150, 200);

    let mut clip_path = Path::new();
    clip_path.push_rect(100.0, 100.0, 800.0, 800.0);
    clip_path.push_rect(300.0, 300.0, 400.0, 400.0);
    clip_path.set_fill_type(FillType::EvenOdd);

    let mut path = Path::new();
    path.push_rect(0.0, 0.0, 1000.0, 1000.0);

    bencher.iter(|| {
        surface.save();
        surface.set_transform(Transform::new(1.0, -0.3, 0.0, 1.0, 0.0, 150.0));
        surface.set_clip_path(&clip_path, aa);
        surface.reset_transform();
        // Do not use draw_rect, because it is very slow by itself.
        surface.draw_path(&path, &paint);
        surface.restore(); // acts as clip reset
    });
}

fn clip_path_skia(bencher: &mut Bencher) {
    do_clip_path_skia(false, bencher);
}

fn clip_path_aa_skia(bencher: &mut Bencher) {
    do_clip_path_skia(true, bencher);
}

fn do_clip_path_raqote(aa: raqote::AntialiasMode, bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    let mut clip_path = {
        let mut pb = PathBuilder::new();
        pb.rect(100.0, 100.0, 800.0, 800.0);
        pb.rect(300.0, 300.0, 400.0, 400.0);
        pb.finish()
    };
    clip_path.winding = Winding::EvenOdd;

    // raqote uses ARGB order.
    let src = Source::from(Color::new(200, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: aa,
    };

    bencher.iter(|| {
        dt.set_transform(&Transform::row_major(1.0, -0.3, 0.0, 1.0, 0.0, 150.0));
        dt.push_clip(&clip_path);
        dt.set_transform(&Transform::default());
        dt.fill_rect(0.0, 0.0, 1000.0, 1000.0, &src, &draw_opt);
        dt.pop_clip();
    });
}

fn clip_path_raqote(bencher: &mut Bencher) {
    do_clip_path_raqote(raqote::AntialiasMode::None, bencher);
}

fn clip_path_aa_raqote(bencher: &mut Bencher) {
    do_clip_path_raqote(raqote::AntialiasMode::Gray, bencher);
}

fn do_clip_path_cairo(aa: cairo::Antialias, bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);
    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);

    bencher.iter(|| {
        cr.set_matrix(Matrix::new(1.0, -0.3, 0.0, 1.0, 0.0, 150.0));
        cr.set_antialias(aa);
        cr.rectangle(100.0, 100.0, 800.0, 800.0);
        cr.rectangle(300.0, 300.0, 400.0, 400.0);
        cr.set_fill_rule(FillRule::EvenOdd);
        cr.clip();

        cr.identity_matrix();
        cr.rectangle(0.0, 0.0, 1000.0, 1000.0);
        cr.set_antialias(cairo::Antialias::None);
        cr.fill();
    });
}

fn clip_path_cairo(bencher: &mut Bencher) {
    do_clip_path_cairo(cairo::Antialias::None, bencher);
}

fn clip_path_aa_cairo(bencher: &mut Bencher) {
    do_clip_path_cairo(cairo::Antialias::Subpixel, bencher);
}

benchmark_group!(clip,
    clip_path_tiny_skia,
    clip_path_skia,
    clip_path_raqote,
    clip_path_cairo,

    clip_path_aa_tiny_skia,
    clip_path_aa_skia,
    clip_path_aa_raqote,
    clip_path_aa_cairo
);
benchmark_main!(clip);
