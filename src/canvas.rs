// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This module is closer to SkDraw than SkCanvas.

use crate::{Pixmap, Transform, Path, Paint, Stroke, Point, Color, Rect};
use crate::{PathBuilder, Pattern, FilterQuality, BlendMode, FillRule, SpreadMode};

use crate::clip::Clip;
use crate::scalar::Scalar;
use crate::stroker::PathStroker;


/// Controls how a pixmap should be blended.
///
/// Like `Paint`, but for `Pixmap`.
#[derive(Copy, Clone, Debug)]
pub struct PixmapPaint {
    /// Pixmap opacity.
    ///
    /// Must be in 0..=1 range.
    ///
    /// Default: 1.0
    pub opacity: f32,

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
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            quality: FilterQuality::Nearest,
        }
    }
}


/// Provides a high-level rendering API.
///
/// Unlike the most of other types, `Canvas` provides an unchecked API.
/// Which means that a drawing command will simply be ignored in case of an error
/// and a caller has no way of checking it.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Canvas {
    /// A pixmap owned by the canvas.
    pub pixmap: Pixmap,

    /// Canvas's transform.
    transform: Transform,

    /// Canvas's clip region.
    clip: Clip,

    /// A path stroker used to cache temporary stroking data.
    stroker: PathStroker,
    stroked_path: Option<Path>,
}

impl From<Pixmap> for Canvas {
    #[inline]
    fn from(pixmap: Pixmap) -> Self {
        Canvas {
            pixmap,
            transform: Transform::identity(),
            clip: Clip::new(),
            stroker: PathStroker::new(),
            stroked_path: None,
        }
    }
}

impl Canvas {
    /// Creates a new canvas.
    ///
    /// A canvas is filled with transparent black by default, aka (0, 0, 0, 0).
    ///
    /// Allocates a new pixmap. Use `Canvas::from(pixmap)` to reuse an existing one.
    ///
    /// Zero size in an error.
    ///
    /// Pixmap's width is limited by i32::MAX/4.
    #[inline]
    pub fn new(width: u32, height: u32) -> Option<Self> {
        Some(Canvas {
            pixmap: Pixmap::new(width, height)?,
            transform: Transform::identity(),
            clip: Clip::new(),
            stroker: PathStroker::new(),
            stroked_path: None,
        })
    }

    /// Translates the canvas.
    #[inline]
    pub fn translate(&mut self, tx: f32, ty: f32) {
        if let Some(ts) = self.transform.pre_translate(tx, ty) {
            self.transform = ts;
        }
    }

    /// Scales the canvas.
    #[inline]
    pub fn scale(&mut self, sx: f32, sy: f32) {
        if let Some(ts) = self.transform.pre_scale(sx, sy) {
            self.transform = ts;
        }
    }

    /// Applies an affine transformation to the canvas.
    #[inline]
    pub fn transform(&mut self, sx: f32, ky: f32, kx: f32, sy: f32, tx: f32, ty: f32) {
        if let Some(ref ts) = Transform::from_row(sx, ky, kx, sy, tx, ty) {
            self.apply_transform(ts);
        }
    }

    // TODO: overload?

    /// Applies an affine transformation to the canvas.
    #[inline]
    pub fn apply_transform(&mut self, ts: &Transform) {
        if let Some(ts) = self.transform.pre_concat(ts) {
            self.transform = ts;
        }
    }

    /// Gets the current canvas transform.
    #[inline]
    pub fn get_transform(&mut self) -> Transform {
        self.transform
    }

    /// Sets the canvas transform.
    #[inline]
    pub fn set_transform(&mut self, ts: Transform) {
        self.transform = ts;
    }

    /// Resets the canvas transform to identity.
    #[inline]
    pub fn reset_transform(&mut self) {
        self.transform = Transform::identity();
    }

    /// Sets a clip rectangle.
    ///
    /// Consecutive calls will replace the previous value.
    ///
    /// Clipping is affected by the current transform.
    pub fn set_clip_rect(&mut self, rect: Rect, anti_alias: bool) {
        self.set_clip_path(&PathBuilder::from_rect(rect), FillRule::Winding, anti_alias);
    }

    /// Sets a clip path.
    ///
    /// Consecutive calls will replace the previous value.
    ///
    /// Clipping is affected by the current transform.
    pub fn set_clip_path(&mut self, path: &Path, fill_type: FillRule, anti_alias: bool) {
        if !self.transform.is_identity() {
            if let Some(ref path) = path.clone().transform(&self.transform) {
                self.clip.set_path(path, self.pixmap.rect(), fill_type, anti_alias);
            }
        } else {
            self.clip.set_path(path, self.pixmap.rect(), fill_type, anti_alias);
        }
    }

    /// Resets the current clip.
    pub fn reset_clip(&mut self) {
        self.clip.clear();
    }

    /// Fills the whole canvas with a color.
    pub fn fill_canvas(&mut self, color: Color) {
        self.pixmap.fill(color);
    }

    /// Fills a path.
    pub fn fill_path(&mut self, path: &Path, paint: &Paint, fill_type: FillRule) {
        self.fill_path_impl(path, paint, fill_type);
    }

    #[inline(always)]
    fn fill_path_impl(&mut self, path: &Path, paint: &Paint, fill_type: FillRule) -> Option<()> {
        if !self.transform.is_identity() {
            let path = path.clone().transform(&self.transform)?;

            let mut paint = paint.clone();
            paint.shader.transform(&self.transform);

            self.pixmap.fill_path(&path, &paint, fill_type, self.clip.as_ref())
        } else {
            self.pixmap.fill_path(path, paint, fill_type, self.clip.as_ref())
        }
    }

    /// Strokes a path.
    ///
    /// Stroking is implemented using two separate algorithms:
    ///
    /// 1. If a stroke width is wider than 1px (after applying the transformation),
    ///    a path will be converted into a stroked path and then filled using `Painter::fill_path`.
    ///    Which means that we have to allocate a separate `Path`, that can be 2-3x larger
    ///    then the original path.
    ///    `Canvas` will reuse this allocation during subsequent strokes.
    /// 2. If a stroke width is thinner than 1px (after applying the transformation),
    ///    we will use hairline stroking, which doesn't involve a separate path allocation.
    ///
    /// Also, if a `stroke` has a dash array, then path will be converted into
    /// a dashed path first and then stroked. Which means a yet another allocation.
    pub fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: &Stroke) {
        self.stroke_path_impl(path, paint, stroke);
    }

    #[inline(always)]
    fn stroke_path_impl(&mut self, path: &Path, paint: &Paint, stroke: &Stroke) -> Option<()> {
        if stroke.width < 0.0 {
            return None;
        }

        let res_scale = PathStroker::compute_resolution_scale(&self.transform);

        let dash_path;
        let path = if let Some(ref dash) = stroke.dash {
            dash_path = crate::dash::dash(path, dash, res_scale)?;
            &dash_path
        } else {
            path
        };

        if let Some(coverage) = treat_as_hairline(&paint, stroke, &self.transform) {
            let mut paint = paint.clone();
            if coverage == 1.0 {
                // No changes to the `paint`.
            } else if paint.blend_mode.should_pre_scale_coverage() {
                // This is the old technique, which we preserve for now so
                // we don't change previous results (testing)
                // the new way seems fine, its just (a tiny bit) different.
                let scale = (coverage * 256.0) as i32;
                let new_alpha = (255 * scale) >> 8;
                paint.shader.apply_opacity(new_alpha as f32 / 255.0);
            }

            if !self.transform.is_identity() {
                paint.shader.transform(&self.transform);

                let path = path.clone().transform(&self.transform)?;
                self.pixmap.stroke_hairline(&path, &paint, stroke.line_cap, self.clip.as_ref())
            } else {
                self.pixmap.stroke_hairline(&path, &paint, stroke.line_cap, self.clip.as_ref())
            }
        } else {
            let mut stroked_path = if let Some(stroked_path) = self.stroked_path.take() {
                self.stroker.stroke_to(&path, stroke, res_scale, stroked_path)
            } else {
                self.stroker.stroke(&path, stroke, res_scale)
            }?;
            stroked_path = stroked_path.transform(&self.transform)?;
            self.stroked_path = Some(stroked_path);

            let path = self.stroked_path.as_ref()?;
            if !self.transform.is_identity() {
                let mut paint = paint.clone();
                paint.shader.transform(&self.transform);

                self.pixmap.fill_path(&path, &paint, FillRule::Winding, self.clip.as_ref())
            } else {
                self.pixmap.fill_path(path, paint, FillRule::Winding, self.clip.as_ref())
            }
        }
    }

    /// Draws a `Pixmap` on top of the current `Pixmap`.
    ///
    /// We basically filling a rectangle with a `pixmap` pattern.
    pub fn draw_pixmap(&mut self, x: i32, y: i32, pixmap: &Pixmap, paint: &PixmapPaint) {
        self.draw_pixmap_impl(x, y, pixmap, paint);
    }

    #[inline(always)]
    fn draw_pixmap_impl(
        &mut self,
        x: i32,
        y: i32,
        pixmap: &Pixmap,
        paint: &PixmapPaint,
    ) -> Option<()> {
        let rect = pixmap.size().to_int_rect(x, y).to_rect();

        // TODO: SkSpriteBlitter
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
            anti_alias: false, // Skia doesn't use it too.
            force_hq_pipeline: false, // Pattern will use hq anyway.
        };

        self.fill_rect_impl(rect, &paint)
    }

    /// Fills a rectangle.
    ///
    /// If there is no transform - uses `Painter::fill_rect`.
    /// Otherwise, it is just a `Canvas::fill_path` with a rectangular path.
    pub fn fill_rect(&mut self, rect: Rect, paint: &Paint) {
        self.fill_rect_impl(rect, paint);
    }

    #[inline(always)]
    fn fill_rect_impl(&mut self, rect: Rect, paint: &Paint) -> Option<()> {
        // TODO: allow translate too
        if self.transform.is_identity() {
            self.pixmap.fill_rect(rect, paint, self.clip.as_ref())
        } else {
            let path = PathBuilder::from_rect(rect);
            self.fill_path_impl(&path, paint, FillRule::Winding)
        }
    }
}

fn treat_as_hairline(paint: &Paint, stroke: &Stroke, ts: &Transform) -> Option<f32> {
    #[inline]
    fn fast_len(p: Point) -> f32 {
        let mut x = p.x.abs();
        let mut y = p.y.abs();
        if x < y {
            std::mem::swap(&mut x, &mut y);
        }

        x + y.half()
    }

    debug_assert!(stroke.width >= 0.0);

    if stroke.width == 0.0 {
        return Some(1.0);
    }

    if !paint.anti_alias {
        return None;
    }

    // We don't care about translate.
    let ts = {
        let (sx, ky, kx, sy, _, _) = ts.get_row();
        Transform::from_row(sx, ky, kx, sy, 0.0, 0.0)?
    };

    // We need to try to fake a thick-stroke with a modulated hairline.
    let mut points = [Point::from_xy(stroke.width, 0.0), Point::from_xy(0.0, stroke.width)];
    ts.map_points(&mut points);

    let len0 = fast_len(points[0]);
    let len1 = fast_len(points[1]);

    if len0 <= 1.0 && len1 <= 1.0 {
        return Some(len0.ave(len1));
    }

    None
}
