/*!
`tiny-skia` is a tiny [Skia](https://skia.org/) subset mostly ported to Rust.
*/

#![doc(html_root_url = "https://docs.rs/tiny-skia/0.2.0")]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

#![allow(clippy::approx_constant)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::float_cmp)]
#![allow(clippy::identity_op)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]

// Must be first, because of macro scope rules.
#[macro_use] mod point;

mod alpha_runs;
mod blend_mode;
mod blitter;
mod checked_geom_ext;
mod color;
mod edge;
mod edge_builder;
mod edge_clipper;
mod fdot6;
mod fixed;
mod floating_point;
mod geometry;
mod line_clipper;
mod math;
mod painter;
mod path;
mod path_builder;
mod path_ops;
mod pixmap;
mod raster_pipeline;
mod scalar;
mod scan;
mod shaders;
mod stroker;
mod wide;

pub use checked_geom::*;

pub use num_ext::NormalizedF32;

pub use blend_mode::BlendMode;
pub use color::{ALPHA_U8_TRANSPARENT, ALPHA_U8_OPAQUE, ALPHA_TRANSPARENT, ALPHA_OPAQUE};
pub use color::{Color, ColorU8, PremultipliedColor, PremultipliedColorU8, AlphaU8};
pub use painter::{Paint, PaintSource, Painter, FillType, SpreadMode};
pub use path::{Path, PathSegment, PathSegmentsIter};
pub use path_builder::PathBuilder;
pub use pixmap::Pixmap;
pub use point::Point;
pub use shaders::{Shader, GradientStop, LinearGradient, RadialGradient};
pub use stroker::{LineCap, LineJoin, StrokeProps, PathStroker};
