// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::*;

use crate::pipeline::RasterPipelineBlitter;
use crate::scalar::Scalar;
use crate::scan;
use crate::stroker::PathStroker;

// 8K is 1 too big, since 8K << supersample == 32768 which is too big for Fixed.
const MAX_DIM: u32 = 8192 - 1;


/// A path filling rule.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FillRule {
    /// Specifies that "inside" is computed by a non-zero sum of signed edge crossings.
    Winding,
    /// Specifies that "inside" is computed by an odd number of edge crossings.
    EvenOdd,
}

impl Default for FillRule {
    #[inline]
    fn default() -> Self {
        FillRule::Winding
    }
}


/// Controls how a shape should be painted.
#[derive(Clone, Debug)]
pub struct Paint<'a> {
    /// A paint shader.
    ///
    /// Default: black color
    pub shader: Shader<'a>,

    /// Paint blending mode.
    ///
    /// Default: SourceOver
    pub blend_mode: BlendMode,

    /// Enables anti-aliased painting.
    ///
    /// Default: false
    pub anti_alias: bool,

    /// Forces the high quality/precision rendering pipeline.
    ///
    /// `tiny-skia`, just like Skia, has two rendering pipelines:
    /// one uses `f32` and another one uses `u16`. `u16` one is usually way faster,
    /// but less precise. Which can lead to slight differences.
    ///
    /// By default, `tiny-skia` will choose the pipeline automatically,
    /// depending on a blending mode and other parameters.
    /// But you can force the high quality one using this flag.
    ///
    /// This feature is especially useful during testing.
    ///
    /// Unlike high quality pipeline, the low quality one doesn't support all
    /// rendering stages, therefore we cannot force it like hq one.
    ///
    /// Default: false
    pub force_hq_pipeline: bool,
}

impl Default for Paint<'_> {
    #[inline]
    fn default() -> Self {
        Paint {
            shader: Shader::SolidColor(Color::BLACK),
            blend_mode: BlendMode::default(),
            anti_alias: false,
            force_hq_pipeline: false,
        }
    }
}

impl<'a> Paint<'a> {
    /// Sets a paint source to a solid color.
    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.shader = Shader::SolidColor(color);
    }

    /// Sets a paint source to a solid color.
    ///
    /// `self.shader = Shader::SolidColor(Color::from_rgba8(50, 127, 150, 200));` shorthand.
    #[inline]
    pub fn set_color_rgba8(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.set_color(Color::from_rgba8(r, g, b, a))
    }

    /// Checks that the paint source is a solid color.
    #[inline]
    pub fn is_solid_color(&self) -> bool {
        matches!(self.shader, Shader::SolidColor(_))
    }
}


impl Pixmap {
    /// Draws a filled rectangle onto the pixmap.
    ///
    /// See [`PixmapMut::fill_rect`](struct.PixmapMut.html#method.fill_rect) for details.
    #[inline]
    pub fn fill_rect(
        &mut self,
        rect: Rect,
        paint: &Paint,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        self.as_mut().fill_rect(rect, paint, transform, clip_mask)
    }

    /// Draws a filled path onto the pixmap.
    ///
    /// See [`PixmapMut::fill_path`](struct.PixmapMut.html#method.fill_path) for details.
    #[inline]
    pub fn fill_path(
        &mut self,
        path: &Path,
        paint: &Paint,
        fill_rule: FillRule,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        self.as_mut().fill_path(path, paint, fill_rule, transform, clip_mask)
    }

    /// Strokes a path.
    ///
    /// See [`PixmapMut::stroke_path`](struct.PixmapMut.html#method.stroke_path) for details.
    #[inline]
    pub fn stroke_path(
        &mut self,
        path: &Path,
        paint: &Paint,
        stroke: &Stroke,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        self.as_mut().stroke_path(path, paint, stroke, transform, clip_mask)
    }

    /// Draws a `Pixmap` on top of the current `Pixmap`.
    ///
    /// See [`PixmapMut::draw_pixmap`](struct.PixmapMut.html#method.draw_pixmap) for details.
    #[inline]
    pub fn draw_pixmap(
        &mut self,
        x: i32,
        y: i32,
        pixmap: PixmapRef,
        paint: &PixmapPaint,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        self.as_mut().draw_pixmap(x, y, pixmap, paint, transform, clip_mask)
    }
}

impl PixmapMut<'_> {
    /// Draws a filled rectangle onto the pixmap.
    ///
    /// This function is usually slower than filling a rectangular path,
    /// but it produces better results. Mainly it doesn't suffer from weird
    /// clipping of horizontal/vertical edges.
    ///
    /// Used mainly to render a pixmap onto a pixmap.
    ///
    /// Returns `None` when there is nothing to fill or in case of a numeric overflow.
    pub fn fill_rect(
        &mut self,
        rect: Rect,
        paint: &Paint,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        if transform.is_identity() {
            // TODO: ignore rects outside the pixmap

            // TODO: draw tiler
            let bbox = rect.round_out();
            if bbox.width() > MAX_DIM || bbox.height() > MAX_DIM {
                return None;
            }

            let clip = self.size().to_screen_int_rect(0, 0);

            let clip_mask = clip_mask.map(|mask| &mask.mask);
            let mut blitter = RasterPipelineBlitter::new(paint, clip_mask, self)?;

            if paint.anti_alias {
                scan::fill_rect_aa(&rect, &clip, &mut blitter)
            } else {
                scan::fill_rect(&rect, &clip, &mut blitter)
            }
        } else {
            let path = PathBuilder::from_rect(rect);
            self.fill_path(&path, paint, FillRule::Winding, transform, clip_mask)
        }
    }

    /// Draws a filled path onto the pixmap.
    ///
    /// Returns `None` when there is nothing to fill or in case of a numeric overflow.
    pub fn fill_path(
        &mut self,
        path: &Path,
        paint: &Paint,
        fill_rule: FillRule,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        if transform.is_identity() {
            // This is sort of similar to SkDraw::drawPath

            // to_rect will fail when bounds' width/height is zero.
            // This is an intended behaviour since the only
            // reason for width/height to be zero is a horizontal/vertical line.
            // And in both cases there is nothing to fill.
            let path_bounds = path.bounds();
            let path_int_bounds = path_bounds.round_out();

            // TODO: ignore paths outside the pixmap

            // TODO: draw tiler
            if path_int_bounds.width() > MAX_DIM || path_int_bounds.height() > MAX_DIM {
                return None;
            }

            if path.is_too_big_for_math() {
                return None;
            }

            let clip_rect = self.size().to_screen_int_rect(0, 0);

            let clip_mask = clip_mask.map(|mask| &mask.mask);
            let mut blitter = RasterPipelineBlitter::new(paint, clip_mask, self)?;

            if paint.anti_alias {
                scan::path_aa::fill_path(path, fill_rule, &clip_rect, &mut blitter)
            } else {
                scan::path::fill_path(path, fill_rule, &clip_rect, &mut blitter)
            }
        } else {
            let path = path.clone().transform(transform)?;

            let mut paint = paint.clone();
            paint.shader.transform(transform);

            self.fill_path(&path, &paint, fill_rule, Transform::identity(), clip_mask)
        }
    }

    // TODO: add dash
    /// Strokes a path.
    ///
    /// Stroking is implemented using two separate algorithms:
    ///
    /// 1. If a stroke width is wider than 1px (after applying the transformation),
    ///    a path will be converted into a stroked path and then filled using `Canvas::fill_path`.
    ///    Which means that we have to allocate a separate `Path`, that can be 2-3x larger
    ///    then the original path.
    ///    `Canvas` will reuse this allocation during subsequent strokes.
    /// 2. If a stroke width is thinner than 1px (after applying the transformation),
    ///    we will use hairline stroking, which doesn't involve a separate path allocation.
    ///
    /// Also, if a `stroke` has a dash array, then path will be converted into
    /// a dashed path first and then stroked. Which means a yet another allocation.
    pub fn stroke_path(
        &mut self,
        path: &Path,
        paint: &Paint,
        stroke: &Stroke,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        if stroke.width < 0.0 {
            return None;
        }

        let res_scale = PathStroker::compute_resolution_scale(&transform);

        let dash_path;
        let path = if let Some(ref dash) = stroke.dash {
            dash_path = crate::dash::dash(path, dash, res_scale)?;
            &dash_path
        } else {
            path
        };

        if let Some(coverage) = treat_as_hairline(&paint, stroke, transform) {
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

            if !transform.is_identity() {
                paint.shader.transform(transform);

                let path = path.clone().transform(transform)?;
                self.stroke_hairline(&path, &paint, stroke.line_cap, clip_mask)
            } else {
                self.stroke_hairline(&path, &paint, stroke.line_cap, clip_mask)
            }
        } else {
            let path = PathStroker::new().stroke(path, stroke, res_scale)?;
            self.fill_path(&path, paint, FillRule::Winding, transform, clip_mask)
        }
    }

    /// A path stroking with subpixel width.
    ///
    /// Should be used when stroke width is <= 1.0
    /// This function doesn't even accept width, which should be regulated via opacity.
    ///
    /// See [`Canvas::stroke_path`] for details.
    ///
    /// [`Canvas::stroke_path`]: struct.Canvas.html#method.stroke_path
    pub(crate) fn stroke_hairline(
        &mut self,
        path: &Path,
        paint: &Paint,
        line_cap: LineCap,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        let clip = self.size().to_screen_int_rect(0, 0);

        let clip_mask = clip_mask.map(|mask| &mask.mask);
        let mut blitter = RasterPipelineBlitter::new(paint, clip_mask, self)?;

        if paint.anti_alias {
            scan::hairline_aa::stroke_path(path, line_cap, &clip, &mut blitter)
        } else {
            scan::hairline::stroke_path(path, line_cap, &clip, &mut blitter)
        }
    }

    /// Draws a `Pixmap` on top of the current `Pixmap`.
    ///
    /// We basically filling a rectangle with a `pixmap` pattern.
    pub fn draw_pixmap(
        &mut self,
        x: i32,
        y: i32,
        pixmap: PixmapRef,
        paint: &PixmapPaint,
        transform: Transform,
        clip_mask: Option<&ClipMask>,
    ) -> Option<()> {
        let rect = pixmap.size().to_int_rect(x, y).to_rect();

        // TODO: SkSpriteBlitter
        // TODO: partially clipped
        // TODO: clipped out

        // Translate pattern as well as bounds.
        let patt_transform = Transform::from_translate(x as f32, y as f32);

        let paint = Paint {
            shader: Pattern::new(
                pixmap,
                SpreadMode::Pad, // Pad, otherwise we will get weird borders overlap.
                paint.quality,
                paint.opacity,
                patt_transform,
            ),
            blend_mode: paint.blend_mode,
            anti_alias: false, // Skia doesn't use it too.
            force_hq_pipeline: false, // Pattern will use hq anyway.
        };

        self.fill_rect(rect, &paint, transform, clip_mask)
    }
}

fn treat_as_hairline(paint: &Paint, stroke: &Stroke, mut ts: Transform) -> Option<f32> {
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
    ts.tx = 0.0;
    ts.ty = 0.0;

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
