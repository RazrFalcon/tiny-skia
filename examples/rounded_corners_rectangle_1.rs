//This example was written by Abdo Mahmoud (Unique-Digital-Resources): https://github.com/Unique-Digital-Resources

use tiny_skia::{Paint, Pixmap, FillRule, Transform, Path, PathBuilder};



fn main() {
    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 127, 0, 200);
    paint.anti_alias = true;

    let mut pixmap = Pixmap::new(1000, 1000).unwrap();



    pixmap.fill_path(
        &rounded_corners_rectangle_draw_1(250.0, 250.0, 800.0, 500.0, 60.0),
        &paint,
        FillRule::EvenOdd,
        Transform::identity(),
        None,
    );

    pixmap.save_png("rounded_corners_rectangle.png").unwrap();
}





fn rounded_corners_rectangle_draw_1(x:f32,y:f32,w:f32,h:f32,mut r:f32) -> Path
{
    if h>w{
        if r > 0.24 * w{
            r = 0.24 * w
        }
    }
    else if h<w {
        if r > 0.24 * h{
            r = 0.24 * h
        }
    }
    else if h==w {
        if r > 0.24 * w{
            r = 0.24 * w
        }
    }

    print!("r = {}",r);

    let mut pb = PathBuilder::new();
    pb.move_to(x+r, y);
    pb.line_to(w-r, y);
    pb.cubic_to(w-(r/2 as f32),y,w,y+(r/2 as f32),w,y+r);
    pb.line_to(w, h-r);
    pb.cubic_to(w,h-(r/2 as f32),w-(r/2 as f32),h,w-r, h);
    pb.line_to(x+r, h);
    pb.cubic_to(x+(r/2 as f32),h,x,h-(r/2 as f32), x ,h-r);
    pb.line_to(x, y+r);
    pb.cubic_to(x,y+(r/2 as f32),x+(r/2 as f32),y, x+r, y);
    pb.close();
    
    
    let path = pb.finish().unwrap();
    return path;
}
