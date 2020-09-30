// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Pixmap, Transform, Path, Paint, StrokeProps, Painter, Point, PathStroker, NormalizedF32};
use crate::{PathBuilder, Pattern, FilterQuality, BlendMode, FillType, Rect, SpreadMode};


/// Controls how a pixmap should be blended.
///
/// Like `Paint`, but for `Pixmap`.
#[derive(Copy, Clone, Debug)]
pub struct PixmapPaint {
    /// Pixmap opacity.
    ///
    /// Default: 1.0
    pub opacity: NormalizedF32,

    /// Pixmap blending mode.
    ///
    /// Default: SourceOver
    pub blend_mode: BlendMode,

    /// Specifies how much filtering to be done when transforming images.
    ///
    /// Default: Nearest
    pub quality: FilterQuality,
}

impl Default for PixmapPaint {
    #[inline]
    fn default() -> Self {
        PixmapPaint {
            opacity: NormalizedF32::ONE,
            blend_mode: BlendMode::default(),
            quality: FilterQuality::Nearest,
        }
    }
}


/// Provides a high-level rendering API.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Canvas {
    /// A pixmap owned by the canvas.
    pub pixmap: Pixmap,
    /// Canvas's transform.
    pub transform: Transform,

    /// A path stroker used to cache temporary stroking data.
    stroker: PathStroker,
}

impl From<Pixmap> for Canvas {
    #[inline]
    fn from(pixmap: Pixmap) -> Self {
        Canvas {
            pixmap,
            transform: Transform::identity(),
            stroker: PathStroker::new(),
        }
    }
}

impl Canvas {
    /// Fills a path.
    pub fn fill_path(&mut self, path: &Path, paint: &Paint) {
        self.fill_path_impl(path, paint);
    }

    #[inline(always)]
    fn fill_path_impl(&mut self, path: &Path, paint: &Paint) -> Option<()> {
        if !self.transform.is_identity() {
            let path = path.clone().transform(&self.transform)?;

            let mut paint = paint.clone();
            paint.shader.transform(&self.transform);

            self.pixmap.fill_path(&path, &paint)
        } else {
            self.pixmap.fill_path(path, paint)
        }
    }

    /// Strokes a path.
    pub fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: StrokeProps) {
        self.stroke_path_impl(path, paint, stroke);
    }

    #[inline(always)]
    fn stroke_path_impl(&mut self, path: &Path, paint: &Paint, mut stroke: StrokeProps) -> Option<()> {
        let mut transformed_paint;
        let transformed_path;
        let (path, paint) = if !self.transform.is_identity() {
            stroke.width *= compute_res_scale_for_stroking(&self.transform);

            transformed_paint = paint.clone();
            transformed_paint.shader.transform(&self.transform);

            transformed_path = path.clone().transform(&self.transform)?;
            (&transformed_path, &transformed_paint)
        } else {
            (path, paint)
        };

        let stroked_path = self.stroker.stroke(&path, stroke)?;

        self.pixmap.fill_path(&stroked_path, paint)
    }

    /// Draws a `Pixmap` on top of the current `Pixmap`.
    pub fn draw_pixmap(&mut self, x: i32, y: i32, pixmap: &Pixmap, paint: &PixmapPaint) {
        self.draw_pixmap_impl(x, y, pixmap, paint);
    }

    #[inline(always)]
    fn draw_pixmap_impl(&mut self, x: i32, y: i32, pixmap: &Pixmap, paint: &PixmapPaint) -> Option<()> {
        // We basically filling a rect with a `pixmap` pattern.

        let rect = pixmap.size().to_int_rect(x, y).to_rect();

        // TODO: partially clipped
        // TODO: clipped out

        // Translate pattern as well as bounds.
        let transform = Transform::from_translate(x as f32, y as f32)?;

        let paint = Paint {
            shader: Pattern::new(
                &pixmap,
                SpreadMode::Pad, // Pad, otherwise we will get weird borders overlap.
                paint.quality,
                paint.opacity,
                transform,
            ),
            blend_mode: paint.blend_mode,
            fill_type: FillType::default(), // Doesn't matter, since we are filling a rectangle.
            anti_alias: false, // Skia doesn't use it too.
            force_hq_pipeline: false, // Pattern will use hq anyway.
        };

        self.fill_rect_impl(&rect, &paint)
    }

    /// Fills a rectangle.
    ///
    /// If there is no transform - uses `Painter::fill_rect`.
    /// Otherwise, it is just a `Canvas::fill_path` with a rectangular path.
    pub fn fill_rect(&mut self, rect: &Rect, paint: &Paint) {
        self.fill_rect_impl(rect, paint);
    }

    #[inline(always)]
    fn fill_rect_impl(&mut self, rect: &Rect, paint: &Paint) -> Option<()> {
        // TODO: allow translate too
        if self.transform.is_identity() {
            self.pixmap.fill_rect(rect, paint)
        } else {
            let bounds = rect.to_bounds()?;
            let path = PathBuilder::from_bounds(bounds);
            self.fill_path_impl(&path, paint)
        }
    }
}

fn compute_res_scale_for_stroking(ts: &Transform) -> f32 {
    let (sx, ky, kx, sy, _,  _) = ts.get_row();
    let sx = Point::from_xy(sx, kx).length();
    let sy = Point::from_xy(ky, sy).length();
    if sx.is_finite() && sy.is_finite() {
        let scale = sx.max(sy);
        if scale > 0.0 {
            return scale;
        }
    }

    1.0
}
