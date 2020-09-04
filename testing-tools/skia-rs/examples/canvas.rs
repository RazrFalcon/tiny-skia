use skia_rs::*;

fn main() {
    let mut surface = Surface::new_rgba_premultiplied(200, 200).unwrap();

    let mut paint = Paint::new();
    paint.set_style(PaintStyle::Fill);
    paint.set_stroke_width(1.0);
    paint.set_anti_alias(false);
    paint.set_shader(&Shader::new_linear_gradient(&LinearGradient {
        start_point: (10.0, 10.0),
        end_point: (190.0, 190.0),
        base: Gradient {
            colors: vec![
                Color::from_rgba(50, 127, 150, 200),
                Color::from_rgba(220, 140, 75, 180),
                Color::from_rgba(40, 180, 55, 160),
            ],
            positions: vec![0.25, 0.50, 0.75],
            tile_mode: TileMode::Clamp,
            transform: Transform::default(),
        },
    }).unwrap());

    let mut path = Path::new();
    path.move_to(10.0, 10.0);
    path.line_to(190.0, 10.0);
    path.line_to(190.0, 190.0);
    path.line_to(10.0, 190.0);
    path.close();
    surface.draw_path(&path, &paint);

    // let start = ((11 * 200) + 35) * 4;
    // println!("{:?}", &surface.data()[start..start+4]);

    surface.save_png("image.png");
}
