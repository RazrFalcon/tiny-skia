use bencher::{benchmark_group, benchmark_main, Bencher};

fn fill_all_tiny_skia(bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();
    let c = Color::from_rgba8(50, 100, 150, 200);
    bencher.iter(|| {
        pixmap.fill(c);
    });
}

fn fill_all_skia(bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();
    let r = 50.0 / 255.0;
    let g = 100.0 / 255.0;
    let b = 150.0 / 255.0;
    let a = 200.0 / 255.0;
    bencher.iter(|| {
        surface.draw_color(r, g, b, a); // TODO: is there a faster way?
    });
}

fn fill_all_raqote(bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);
    bencher.iter(|| {
        dt.clear(SolidSource { r: 50, g: 100, b: 150, a: 200 });
    });
}

fn fill_all_cairo(bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);
    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);

    bencher.iter(|| {
        cr.paint(); // TODO: is there a faster way?
    });
}

benchmark_group!(fill_all,
    fill_all_tiny_skia,
    fill_all_skia,
    fill_all_raqote,
    fill_all_cairo
);
benchmark_main!(fill_all);
