use bencher::{benchmark_group, benchmark_main, Bencher};

fn decode_rgb(bencher: &mut Bencher) {
    let data = std::fs::read("../tests/images/pngs/rgb.png").unwrap();
    bencher.iter(|| {
        let _ = tiny_skia::Pixmap::decode_png(&data).unwrap();
    });
}

fn decode_rgba(bencher: &mut Bencher) {
    let data = std::fs::read("../tests/images/pngs/rgba.png").unwrap();
    bencher.iter(|| {
        let _ = tiny_skia::Pixmap::decode_png(&data).unwrap();
    });
}

// Just a PNG decoding without preprocessing
// to see how much overhead our code has.
fn decode_raw_rgb(bencher: &mut Bencher) {
    let data = std::fs::read("../tests/images/pngs/rgb.png").unwrap();
    let mut img_data = vec![0; 30000];
    bencher.iter(|| {
        let decoder = png::Decoder::new(data.as_slice());
        let (_, mut reader) = decoder.read_info().unwrap();
        let _ = reader.next_frame(&mut img_data).unwrap();
    });
}

fn decode_raw_rgba(bencher: &mut Bencher) {
    let data = std::fs::read("../tests/images/pngs/rgba.png").unwrap();
    let mut img_data = vec![0; 40000];
    bencher.iter(|| {
        let decoder = png::Decoder::new(data.as_slice());
        let (_, mut reader) = decoder.read_info().unwrap();
        let _ = reader.next_frame(&mut img_data).unwrap();
    });
}

fn encode_rgba(bencher: &mut Bencher) {
    let pixmap = tiny_skia::Pixmap::load_png("../tests/images/pngs/rgba.png").unwrap();
    bencher.iter(|| {
        let _ = pixmap.encode_png().unwrap();
    });
}

fn encode_raw_rgba(bencher: &mut Bencher) {
    let pixmap = tiny_skia::Pixmap::load_png("../tests/images/pngs/rgba.png").unwrap();
    bencher.iter(|| {
        let mut data = Vec::new();

        let mut encoder = png::Encoder::new(&mut data, pixmap.width(), pixmap.height());
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(pixmap.data()).unwrap();
    });
}

benchmark_group!(decode, decode_rgb, decode_rgba, decode_raw_rgb, decode_raw_rgba);
benchmark_group!(encode, encode_rgba, encode_raw_rgba);
benchmark_main!(decode, encode);
