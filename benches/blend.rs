use bencher::{benchmark_group, benchmark_main, Bencher};

fn fill_tiny_skia(blend_mode: tiny_skia::BlendMode, bencher: &mut Bencher) {
    use tiny_skia::*;

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();

    let paint1 = Paint::default()
        .set_color_rgba8(50, 127, 150, 200)
        .set_blend_mode(BlendMode::SourceOver);

    let paint2 = Paint::default()
        .set_color_rgba8(220, 140, 75, 180)
        .set_blend_mode(blend_mode); // <-- variable

    let path1 = {
        let mut pb = PathBuilder::new();
        pb.move_to(60.0, 60.0);
        pb.line_to(160.0, 940.0);
        pb.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
        pb.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
        pb.close();
        pb.finish().unwrap()
    };

    let path2 = {
        let mut pb = PathBuilder::new();
        pb.move_to(940.0, 60.0);
        pb.line_to(840.0, 940.0);
        pb.cubic_to(620.0, 840.0, 340.0, 800.0, 60.0, 800.0);
        pb.cubic_to(260.0, 460.0, 560.0, 160.0, 940.0, 60.0);
        pb.close();
        pb.finish().unwrap()
    };

    pixmap.fill_path(&path1, &paint1);

    bencher.iter(|| {
        pixmap.fill_path(&path2, &paint2);
    });
}

fn clear_tiny_skia(bencher: &mut Bencher)               { fill_tiny_skia(tiny_skia::BlendMode::Clear, bencher); }
fn source_tiny_skia(bencher: &mut Bencher)              { fill_tiny_skia(tiny_skia::BlendMode::Source, bencher); }
fn destination_tiny_skia(bencher: &mut Bencher)         { fill_tiny_skia(tiny_skia::BlendMode::Destination, bencher); }
fn source_over_tiny_skia(bencher: &mut Bencher)         { fill_tiny_skia(tiny_skia::BlendMode::SourceOver, bencher); }
fn destination_over_tiny_skia(bencher: &mut Bencher)    { fill_tiny_skia(tiny_skia::BlendMode::DestinationOver, bencher); }
fn source_in_tiny_skia(bencher: &mut Bencher)           { fill_tiny_skia(tiny_skia::BlendMode::SourceIn, bencher); }
fn destination_in_tiny_skia(bencher: &mut Bencher)      { fill_tiny_skia(tiny_skia::BlendMode::DestinationIn, bencher); }
fn source_out_tiny_skia(bencher: &mut Bencher)          { fill_tiny_skia(tiny_skia::BlendMode::SourceOut, bencher); }
fn destination_out_tiny_skia(bencher: &mut Bencher)     { fill_tiny_skia(tiny_skia::BlendMode::DestinationOut, bencher); }
fn source_atop_tiny_skia(bencher: &mut Bencher)         { fill_tiny_skia(tiny_skia::BlendMode::SourceAtop, bencher); }
fn destination_atop_tiny_skia(bencher: &mut Bencher)    { fill_tiny_skia(tiny_skia::BlendMode::DestinationAtop, bencher); }
fn xor_tiny_skia(bencher: &mut Bencher)                 { fill_tiny_skia(tiny_skia::BlendMode::Xor, bencher); }
fn plus_tiny_skia(bencher: &mut Bencher)                { fill_tiny_skia(tiny_skia::BlendMode::Plus, bencher); }
fn modulate_tiny_skia(bencher: &mut Bencher)            { fill_tiny_skia(tiny_skia::BlendMode::Modulate, bencher); }
fn screen_tiny_skia(bencher: &mut Bencher)              { fill_tiny_skia(tiny_skia::BlendMode::Screen, bencher); }
fn overlay_tiny_skia(bencher: &mut Bencher)             { fill_tiny_skia(tiny_skia::BlendMode::Overlay, bencher); }
fn darken_tiny_skia(bencher: &mut Bencher)              { fill_tiny_skia(tiny_skia::BlendMode::Darken, bencher); }
fn lighten_tiny_skia(bencher: &mut Bencher)             { fill_tiny_skia(tiny_skia::BlendMode::Lighten, bencher); }
fn color_dodge_tiny_skia(bencher: &mut Bencher)         { fill_tiny_skia(tiny_skia::BlendMode::ColorDodge, bencher); }
fn color_burn_tiny_skia(bencher: &mut Bencher)          { fill_tiny_skia(tiny_skia::BlendMode::ColorBurn, bencher); }
fn hard_light_tiny_skia(bencher: &mut Bencher)          { fill_tiny_skia(tiny_skia::BlendMode::HardLight, bencher); }
fn soft_light_tiny_skia(bencher: &mut Bencher)          { fill_tiny_skia(tiny_skia::BlendMode::SoftLight, bencher); }
fn difference_tiny_skia(bencher: &mut Bencher)          { fill_tiny_skia(tiny_skia::BlendMode::Difference, bencher); }
fn exclusion_tiny_skia(bencher: &mut Bencher)           { fill_tiny_skia(tiny_skia::BlendMode::Exclusion, bencher); }
fn multiply_tiny_skia(bencher: &mut Bencher)            { fill_tiny_skia(tiny_skia::BlendMode::Multiply, bencher); }
fn hue_tiny_skia(bencher: &mut Bencher)                 { fill_tiny_skia(tiny_skia::BlendMode::Hue, bencher); }
fn saturation_tiny_skia(bencher: &mut Bencher)          { fill_tiny_skia(tiny_skia::BlendMode::Saturation, bencher); }
fn color_tiny_skia(bencher: &mut Bencher)               { fill_tiny_skia(tiny_skia::BlendMode::Color, bencher); }
fn luminosity_tiny_skia(bencher: &mut Bencher)          { fill_tiny_skia(tiny_skia::BlendMode::Luminosity, bencher); }

fn fill_skia(mode: skia_rs::BlendMode, bencher: &mut Bencher) {
    use skia_rs::*;

    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let mut paint1 = Paint::new();
    paint1.set_color(50, 127, 150, 200);
    paint1.set_blend_mode(BlendMode::SourceOver);

    let mut paint2 = Paint::new();
    paint2.set_color(220, 140, 75, 180);
    paint2.set_blend_mode(mode); // <-- variable

    let path1 = {
        let mut path = Path::new();
        path.move_to(60.0, 60.0);
        path.line_to(160.0, 940.0);
        path.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
        path.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
        path.close();
        path
    };

    let path2 = {
        let mut path = Path::new();
        path.move_to(940.0, 60.0);
        path.line_to(840.0, 940.0);
        path.cubic_to(620.0, 840.0, 340.0, 800.0, 60.0, 800.0);
        path.cubic_to(260.0, 460.0, 560.0, 160.0, 940.0, 60.0);
        path.close();
        path
    };

    surface.draw_path(&path1, &paint1);

    bencher.iter(|| {
        surface.draw_path(&path2, &paint2);
    });
}

fn clear_skia(bencher: &mut Bencher)               { fill_skia(skia_rs::BlendMode::Clear, bencher); }
fn source_skia(bencher: &mut Bencher)              { fill_skia(skia_rs::BlendMode::Source, bencher); }
fn destination_skia(bencher: &mut Bencher)         { fill_skia(skia_rs::BlendMode::Destination, bencher); }
fn source_over_skia(bencher: &mut Bencher)         { fill_skia(skia_rs::BlendMode::SourceOver, bencher); }
fn destination_over_skia(bencher: &mut Bencher)    { fill_skia(skia_rs::BlendMode::DestinationOver, bencher); }
fn source_in_skia(bencher: &mut Bencher)           { fill_skia(skia_rs::BlendMode::SourceIn, bencher); }
fn destination_in_skia(bencher: &mut Bencher)      { fill_skia(skia_rs::BlendMode::DestinationIn, bencher); }
fn source_out_skia(bencher: &mut Bencher)          { fill_skia(skia_rs::BlendMode::SourceOut, bencher); }
fn destination_out_skia(bencher: &mut Bencher)     { fill_skia(skia_rs::BlendMode::DestinationOut, bencher); }
fn source_atop_skia(bencher: &mut Bencher)         { fill_skia(skia_rs::BlendMode::SourceATop, bencher); }
fn destination_atop_skia(bencher: &mut Bencher)    { fill_skia(skia_rs::BlendMode::DestinationATop, bencher); }
fn xor_skia(bencher: &mut Bencher)                 { fill_skia(skia_rs::BlendMode::Xor, bencher); }
fn plus_skia(bencher: &mut Bencher)                { fill_skia(skia_rs::BlendMode::Plus, bencher); }
fn modulate_skia(bencher: &mut Bencher)            { fill_skia(skia_rs::BlendMode::Modulate, bencher); }
fn screen_skia(bencher: &mut Bencher)              { fill_skia(skia_rs::BlendMode::Screen, bencher); }
fn overlay_skia(bencher: &mut Bencher)             { fill_skia(skia_rs::BlendMode::Overlay, bencher); }
fn darken_skia(bencher: &mut Bencher)              { fill_skia(skia_rs::BlendMode::Darken, bencher); }
fn lighten_skia(bencher: &mut Bencher)             { fill_skia(skia_rs::BlendMode::Lighten, bencher); }
fn color_dodge_skia(bencher: &mut Bencher)         { fill_skia(skia_rs::BlendMode::ColorDodge, bencher); }
fn color_burn_skia(bencher: &mut Bencher)          { fill_skia(skia_rs::BlendMode::ColorBurn, bencher); }
fn hard_light_skia(bencher: &mut Bencher)          { fill_skia(skia_rs::BlendMode::HardLight, bencher); }
fn soft_light_skia(bencher: &mut Bencher)          { fill_skia(skia_rs::BlendMode::SoftLight, bencher); }
fn difference_skia(bencher: &mut Bencher)          { fill_skia(skia_rs::BlendMode::Difference, bencher); }
fn exclusion_skia(bencher: &mut Bencher)           { fill_skia(skia_rs::BlendMode::Exclusion, bencher); }
fn multiply_skia(bencher: &mut Bencher)            { fill_skia(skia_rs::BlendMode::Multiply, bencher); }
fn hue_skia(bencher: &mut Bencher)                 { fill_skia(skia_rs::BlendMode::Hue, bencher); }
fn saturation_skia(bencher: &mut Bencher)          { fill_skia(skia_rs::BlendMode::Saturation, bencher); }
fn color_skia(bencher: &mut Bencher)               { fill_skia(skia_rs::BlendMode::Color, bencher); }
fn luminosity_skia(bencher: &mut Bencher)          { fill_skia(skia_rs::BlendMode::Luminosity, bencher); }

fn fill_raqote(blend_mode: raqote::BlendMode, bencher: &mut Bencher) {
    use raqote::*;

    let mut dt = DrawTarget::new(1000, 1000);

    let path1 = {
        let mut pb = PathBuilder::new();
        pb.move_to(60.0, 60.0);
        pb.line_to(160.0, 940.0);
        pb.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
        pb.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
        pb.close();
        pb.finish()
    };

    let path2 = {
        let mut pb = PathBuilder::new();
        pb.move_to(940.0, 60.0);
        pb.line_to(840.0, 940.0);
        pb.cubic_to(620.0, 840.0, 340.0, 800.0, 60.0, 800.0);
        pb.cubic_to(260.0, 460.0, 560.0, 160.0, 940.0, 60.0);
        pb.close();
        pb.finish()
    };

    // raqote uses ARGB order.
    let src1 = Source::from(Color::new(200, 50, 127, 150));
    let src2 = Source::from(Color::new(180, 220, 140, 75));

    let draw_opt1 = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::None,
    };

    let draw_opt2 = DrawOptions {
        blend_mode, // <-- variable
        alpha: 1.0,
        antialias: AntialiasMode::None,
    };

    dt.fill(&path1, &src1, &draw_opt1);

    bencher.iter(|| {
        dt.fill(&path2, &src2, &draw_opt2);
    });
}

fn clear_raqote(bencher: &mut Bencher)               { fill_raqote(raqote::BlendMode::Clear, bencher); }
fn source_raqote(bencher: &mut Bencher)              { fill_raqote(raqote::BlendMode::Src, bencher); }
fn destination_raqote(bencher: &mut Bencher)         { fill_raqote(raqote::BlendMode::Dst, bencher); }
fn source_over_raqote(bencher: &mut Bencher)         { fill_raqote(raqote::BlendMode::SrcOver, bencher); }
fn destination_over_raqote(bencher: &mut Bencher)    { fill_raqote(raqote::BlendMode::DstOver, bencher); }
fn source_in_raqote(bencher: &mut Bencher)           { fill_raqote(raqote::BlendMode::SrcIn, bencher); }
fn destination_in_raqote(bencher: &mut Bencher)      { fill_raqote(raqote::BlendMode::DstIn, bencher); }
fn source_out_raqote(bencher: &mut Bencher)          { fill_raqote(raqote::BlendMode::SrcOut, bencher); }
fn destination_out_raqote(bencher: &mut Bencher)     { fill_raqote(raqote::BlendMode::DstOut, bencher); }
fn source_atop_raqote(bencher: &mut Bencher)         { fill_raqote(raqote::BlendMode::SrcAtop, bencher); }
fn destination_atop_raqote(bencher: &mut Bencher)    { fill_raqote(raqote::BlendMode::DstAtop, bencher); }
fn xor_raqote(bencher: &mut Bencher)                 { fill_raqote(raqote::BlendMode::Xor, bencher); }
fn plus_raqote(bencher: &mut Bencher)                { fill_raqote(raqote::BlendMode::Add, bencher); }
// fn modulate_raqote(bencher: &mut Bencher)            { fill_raqote(raqote::BlendMode::Modulate, bencher); } // TODO: missing?
fn screen_raqote(bencher: &mut Bencher)              { fill_raqote(raqote::BlendMode::Screen, bencher); }
fn overlay_raqote(bencher: &mut Bencher)             { fill_raqote(raqote::BlendMode::Overlay, bencher); }
fn darken_raqote(bencher: &mut Bencher)              { fill_raqote(raqote::BlendMode::Darken, bencher); }
fn lighten_raqote(bencher: &mut Bencher)             { fill_raqote(raqote::BlendMode::Lighten, bencher); }
fn color_dodge_raqote(bencher: &mut Bencher)         { fill_raqote(raqote::BlendMode::ColorDodge, bencher); }
fn color_burn_raqote(bencher: &mut Bencher)          { fill_raqote(raqote::BlendMode::ColorBurn, bencher); }
fn hard_light_raqote(bencher: &mut Bencher)          { fill_raqote(raqote::BlendMode::HardLight, bencher); }
fn soft_light_raqote(bencher: &mut Bencher)          { fill_raqote(raqote::BlendMode::SoftLight, bencher); }
fn difference_raqote(bencher: &mut Bencher)          { fill_raqote(raqote::BlendMode::Difference, bencher); }
fn exclusion_raqote(bencher: &mut Bencher)           { fill_raqote(raqote::BlendMode::Exclusion, bencher); }
fn multiply_raqote(bencher: &mut Bencher)            { fill_raqote(raqote::BlendMode::Multiply, bencher); }
fn hue_raqote(bencher: &mut Bencher)                 { fill_raqote(raqote::BlendMode::Hue, bencher); }
fn saturation_raqote(bencher: &mut Bencher)          { fill_raqote(raqote::BlendMode::Saturation, bencher); }
fn color_raqote(bencher: &mut Bencher)               { fill_raqote(raqote::BlendMode::Color, bencher); }
fn luminosity_raqote(bencher: &mut Bencher)          { fill_raqote(raqote::BlendMode::Luminosity, bencher); }

fn fill_cairo(op: cairo::Operator, bencher: &mut Bencher) {
    use cairo::*;

    let surface = ImageSurface::create(Format::ARgb32, 1000, 1000).unwrap();

    let cr = Context::new(&surface);

    cr.move_to(60.0, 60.0);
    cr.line_to(160.0, 940.0);
    cr.curve_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
    cr.curve_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
    cr.close_path();

    cr.set_source_rgba(50.0 / 255.0, 127.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0);
    cr.set_antialias(Antialias::None);
    cr.set_fill_rule(FillRule::Winding);
    cr.set_operator(Operator::Over);
    cr.fill();

    cr.move_to(940.0, 60.0);
    cr.line_to(840.0, 940.0);
    cr.curve_to(620.0, 840.0, 340.0, 800.0, 60.0, 800.0);
    cr.curve_to(260.0, 460.0, 560.0, 160.0, 940.0, 60.0);
    cr.close_path();

    cr.set_source_rgba(220.0 / 255.0, 140.0 / 255.0, 75.0 / 255.0, 180.0 / 255.0);
    cr.set_antialias(Antialias::None);
    cr.set_fill_rule(FillRule::Winding);
    cr.set_operator(op); // <-- variable

    bencher.iter(|| {
        cr.fill_preserve();
    });
}

fn clear_cairo(bencher: &mut Bencher)               { fill_cairo(cairo::Operator::Clear, bencher); }
fn source_cairo(bencher: &mut Bencher)              { fill_cairo(cairo::Operator::Source, bencher); }
fn destination_cairo(bencher: &mut Bencher)         { fill_cairo(cairo::Operator::Dest, bencher); }
fn source_over_cairo(bencher: &mut Bencher)         { fill_cairo(cairo::Operator::Over, bencher); }
fn destination_over_cairo(bencher: &mut Bencher)    { fill_cairo(cairo::Operator::DestOver, bencher); }
fn source_in_cairo(bencher: &mut Bencher)           { fill_cairo(cairo::Operator::In, bencher); }
fn destination_in_cairo(bencher: &mut Bencher)      { fill_cairo(cairo::Operator::DestIn, bencher); }
fn source_out_cairo(bencher: &mut Bencher)          { fill_cairo(cairo::Operator::Out, bencher); }
fn destination_out_cairo(bencher: &mut Bencher)     { fill_cairo(cairo::Operator::DestOut, bencher); }
fn source_atop_cairo(bencher: &mut Bencher)         { fill_cairo(cairo::Operator::Atop, bencher); }
fn destination_atop_cairo(bencher: &mut Bencher)    { fill_cairo(cairo::Operator::DestAtop, bencher); }
fn xor_cairo(bencher: &mut Bencher)                 { fill_cairo(cairo::Operator::Xor, bencher); }
fn plus_cairo(bencher: &mut Bencher)                { fill_cairo(cairo::Operator::Add, bencher); }
// fn modulate_cairo(bencher: &mut Bencher)            { fill_cairo(cairo::Operator::Modulate, bencher); } // TODO: missing?
fn screen_cairo(bencher: &mut Bencher)              { fill_cairo(cairo::Operator::Screen, bencher); }
fn overlay_cairo(bencher: &mut Bencher)             { fill_cairo(cairo::Operator::Overlay, bencher); }
fn darken_cairo(bencher: &mut Bencher)              { fill_cairo(cairo::Operator::Darken, bencher); }
fn lighten_cairo(bencher: &mut Bencher)             { fill_cairo(cairo::Operator::Lighten, bencher); }
fn color_dodge_cairo(bencher: &mut Bencher)         { fill_cairo(cairo::Operator::ColorDodge, bencher); }
fn color_burn_cairo(bencher: &mut Bencher)          { fill_cairo(cairo::Operator::ColorBurn, bencher); }
fn hard_light_cairo(bencher: &mut Bencher)          { fill_cairo(cairo::Operator::HardLight, bencher); }
fn soft_light_cairo(bencher: &mut Bencher)          { fill_cairo(cairo::Operator::SoftLight, bencher); }
fn difference_cairo(bencher: &mut Bencher)          { fill_cairo(cairo::Operator::Difference, bencher); }
fn exclusion_cairo(bencher: &mut Bencher)           { fill_cairo(cairo::Operator::Exclusion, bencher); }
fn multiply_cairo(bencher: &mut Bencher)            { fill_cairo(cairo::Operator::Multiply, bencher); }
fn hue_cairo(bencher: &mut Bencher)                 { fill_cairo(cairo::Operator::HslHue, bencher); }
fn saturation_cairo(bencher: &mut Bencher)          { fill_cairo(cairo::Operator::HslSaturation, bencher); }
fn color_cairo(bencher: &mut Bencher)               { fill_cairo(cairo::Operator::HslColor, bencher); }
fn luminosity_cairo(bencher: &mut Bencher)          { fill_cairo(cairo::Operator::HslLuminosity, bencher); }


benchmark_group!(benches,
    clear_tiny_skia,
    source_tiny_skia,
    destination_tiny_skia,
    source_over_tiny_skia,
    destination_over_tiny_skia,
    source_in_tiny_skia,
    destination_in_tiny_skia,
    source_out_tiny_skia,
    destination_out_tiny_skia,
    source_atop_tiny_skia,
    destination_atop_tiny_skia,
    xor_tiny_skia,
    plus_tiny_skia,
    modulate_tiny_skia,
    screen_tiny_skia,
    overlay_tiny_skia,
    darken_tiny_skia,
    lighten_tiny_skia,
    color_dodge_tiny_skia,
    color_burn_tiny_skia,
    hard_light_tiny_skia,
    soft_light_tiny_skia,
    difference_tiny_skia,
    exclusion_tiny_skia,
    multiply_tiny_skia,
    hue_tiny_skia,
    saturation_tiny_skia,
    color_tiny_skia,
    luminosity_tiny_skia,

    clear_skia,
    source_skia,
    destination_skia,
    source_over_skia,
    destination_over_skia,
    source_in_skia,
    destination_in_skia,
    source_out_skia,
    destination_out_skia,
    source_atop_skia,
    destination_atop_skia,
    xor_skia,
    plus_skia,
    modulate_skia,
    screen_skia,
    overlay_skia,
    darken_skia,
    lighten_skia,
    color_dodge_skia,
    color_burn_skia,
    hard_light_skia,
    soft_light_skia,
    difference_skia,
    exclusion_skia,
    multiply_skia,
    hue_skia,
    saturation_skia,
    color_skia,
    luminosity_skia,

    clear_raqote,
    source_raqote,
    destination_raqote,
    source_over_raqote,
    destination_over_raqote,
    source_in_raqote,
    destination_in_raqote,
    source_out_raqote,
    destination_out_raqote,
    source_atop_raqote,
    destination_atop_raqote,
    xor_raqote,
    plus_raqote,
    // modulate_raqote,
    screen_raqote,
    overlay_raqote,
    darken_raqote,
    lighten_raqote,
    color_dodge_raqote,
    color_burn_raqote,
    hard_light_raqote,
    soft_light_raqote,
    difference_raqote,
    exclusion_raqote,
    multiply_raqote,
    hue_raqote,
    saturation_raqote,
    color_raqote,
    luminosity_raqote,

    clear_cairo,
    source_cairo,
    destination_cairo,
    source_over_cairo,
    destination_over_cairo,
    source_in_cairo,
    destination_in_cairo,
    source_out_cairo,
    destination_out_cairo,
    source_atop_cairo,
    destination_atop_cairo,
    xor_cairo,
    plus_cairo,
    // modulate_cairo,
    screen_cairo,
    overlay_cairo,
    darken_cairo,
    lighten_cairo,
    color_dodge_cairo,
    color_burn_cairo,
    hard_light_cairo,
    soft_light_cairo,
    difference_cairo,
    exclusion_cairo,
    multiply_cairo,
    hue_cairo,
    saturation_cairo,
    color_cairo,
    luminosity_cairo,
);

benchmark_main!(benches);
