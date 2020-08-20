/*!
`tiny-skia` is a tiny [Skia](https://skia.org/) subset mostly ported to Rust.
*/

#![doc(html_root_url = "https://docs.rs/tiny-skia/0.2.0")]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

mod blend_mode;
mod blitter;
mod checked_geom_ext;
mod color;
mod edge;
mod edge_builder;
mod fdot6;
mod fixed;
mod floating_point;
mod geometry;
mod math;
mod painter;
mod path;
mod path_builder;
mod pixmap;
mod raster_pipeline;
mod raster_pipeline_blitter;
mod scan;

pub use checked_geom::*;

pub use num_ext::NormalizedF32;

pub use blend_mode::BlendMode;
pub use color::{ALPHA_U8_TRANSPARENT, ALPHA_U8_OPAQUE, ALPHA_TRANSPARENT, ALPHA_OPAQUE};
pub use color::{Color, ColorU8, PremultipliedColor, PremultipliedColorU8, AlphaU8};
pub use painter::{Paint, Painter, FillType};
pub use path::{Path, PathSegment, PathSegmentsIter};
pub use path_builder::PathBuilder;
pub use pixmap::Pixmap;
