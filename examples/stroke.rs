use tiny_skia::*;

fn main() {
    let mut pixmap = Pixmap::new(500, 500).unwrap();

    let now = std::time::Instant::now();

    stroke_simple(&mut pixmap);
    stroke_preserve(&mut pixmap);

    println!("Rendered in {:.2}ms", now.elapsed().as_micros() as f64 / 1000.0);

    pixmap.save_png("image.png").unwrap();
}

// In tiny-skia, there is no draw_path method that can fill and/or stroke a path.
// Instead, to stroke a path you should convert it into a stroke outline first
// and then fill the outline as usual.
fn stroke_simple(pixmap: &mut Pixmap) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(0xDD, 0, 0, 0xAA);
    paint.anti_alias = true;

    let path = {
        let mut pb = PathBuilder::new();
        pb.move_to(45.0, 360.0);
        pb.line_to(220.0, 260.0);
        pb.line_to(455.0, 260.0);
        pb.line_to(280.0, 360.0);
        pb.close();
        pb.finish().unwrap()
    };

    let mut props = StrokeProps::default();
    props.width = 4.0;
    let stroked_path = path.stroke(props).unwrap();

    pixmap.fill_path(&stroked_path, &paint);
}

// The stroking algorithm will use multiple temporary Path buffers.
// To reuse the allocated memory, you can use the PathStroker directly.
fn stroke_preserve(pixmap: &mut Pixmap) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 0xDD, 0, 0xAA);
    paint.anti_alias = true;

    let mut props = StrokeProps::default();
    props.width = 4.0;

    let mut stroker = PathStroker::new();

    let path = {
        let mut pb = PathBuilder::new();
        pb.move_to(45.0, 300.0);
        pb.line_to(220.0, 200.0);
        pb.line_to(455.0, 200.0);
        pb.line_to(280.0, 300.0);
        pb.close();
        pb.finish().unwrap()
    };

    let stroked_path = stroker.stroke(&path, props).unwrap();
    pixmap.fill_path(&stroked_path, &paint);

    // All path building/stroking code below will not allocate new memory.

    // Reuse path.
    let path = {
        let mut pb = path.clear();
        pb.move_to(45.0, 240.0);
        pb.line_to(220.0, 140.0);
        pb.line_to(455.0, 140.0);
        pb.line_to(280.0, 240.0);
        pb.close();
        pb.finish().unwrap()
    };

    // Reuse allocated stroker buffers and stroked path.
    let stroked_path = stroker.stroke_to(&path, props, stroked_path).unwrap();

    paint.set_color_rgba8(0, 0, 0xDD, 0xAA);
    pixmap.fill_path(&stroked_path, &paint);
}
