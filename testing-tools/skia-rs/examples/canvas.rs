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

fn main() {
    let mut surface = Surface::new_rgba_premultiplied(1000, 1000).unwrap();

    let triangle = crate_triangle();
    let shader = Shader::new_from_surface_image(
        &triangle,
        Transform::new(1.5, -0.4, 0.0, -0.8, 5.0, 1.0),
        FilterQuality::High,
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
    surface.draw_path(&path, &paint);

    // let start = ((11 * 200) + 35) * 4;
    // println!("{:?}", &surface.data()[start..start+4]);

    surface.save_png("image.png");
}
