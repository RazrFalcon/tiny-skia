// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Pixmap, Path, Color, BlendMode, Shader, Rect};

use crate::scan;
use crate::raster_pipeline::{ContextStorage, RasterPipelineBlitter};

// 8K is 1 too big, since 8K << supersample == 32768 which is too big for Fixed.
const MAX_DIM: u32 = 8192 - 1;


/// A path filling type.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FillType {
    /// Specifies that "inside" is computed by a non-zero sum of signed edge crossings.
    Winding,
    /// Specifies that "inside" is computed by an odd number of edge crossings.
    EvenOdd,
}

impl Default for FillType {
    #[inline]
    fn default() -> Self {
        FillType::Winding
    }
}


/// Controls how a path should be filled.
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

    /// A path filling type.
    ///
    /// Default: Winding
    pub fill_type: FillType,

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
            fill_type: FillType::default(),
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
        match self.shader {
            Shader::SolidColor(_) => true,
            _ => false,
        }
    }
}


/// Provides a low-level rendering API.
pub trait Painter {
    /// Fills the entire pixmap with a specified color.
    ///
    /// This is essentially a memset, therefore it's very fast.
    fn fill(&mut self, color: Color);

    /// Draws a filled rectangle onto the pixmap.
    ///
    /// This function is usually slower than filling a rectangular path,
    /// but it produces better results. Mainly it doesn't suffer from weird
    /// clipping of horizontal/vertical edges.
    ///
    /// Used mainly to render a pixmap onto a pixmap.
    ///
    /// Returns `None` when there is nothing to fill or in case of a numeric overflow.
    fn fill_rect(&mut self, rect: &Rect, paint: &Paint) -> Option<()>;

    /// Draws a filled path onto the pixmap.
    ///
    /// Returns `None` when there is nothing to fill or in case of a numeric overflow.
    fn fill_path(&mut self, path: &Path, paint: &Paint) -> Option<()>;
}

impl Painter for Pixmap {
    fn fill(&mut self, color: Color) {
        // TODO: use memset for colors with even components, like 0 0 0 0

        let c = color.premultiply().to_color_u8();
        for p in self.pixels_mut() {
            *p = c;
        }
    }

    fn fill_rect(&mut self, rect: &Rect, paint: &Paint) -> Option<()> {
        // TODO: ignore rects outside the pixmap

        // TODO: draw tiler
        let bbox = rect.round_out();
        if bbox.width() > MAX_DIM || bbox.height() > MAX_DIM {
            return None;
        }

        let clip = self.size().to_screen_int_rect(0, 0);

        let mut ctx_storage = ContextStorage::new();
        let mut blitter = RasterPipelineBlitter::new(paint, &mut ctx_storage, self)?;

        if paint.anti_alias {
            scan::fill_rect_aa(rect, &clip, &mut blitter)
        } else {
            scan::fill_rect(rect, &clip, &mut blitter)
        }
    }

    fn fill_path(&mut self, path: &Path, paint: &Paint) -> Option<()> {
        // This is sort of similar to SkDraw::drawPath

        // to_rect will fail when bounds' width/height is zero.
        // This is an intended behaviour since the only
        // reason for width/height to be zero is a horizontal/vertical line.
        // And in both cases there is nothing to fill.
        let path_bounds = path.bounds().to_rect()?;
        let path_int_bounds = path_bounds.round_out();

        // TODO: ignore paths outside the pixmap

        // TODO: draw tiler
        if path_int_bounds.width() > MAX_DIM || path_int_bounds.height() > MAX_DIM {
            return None;
        }

        if path.is_too_big_for_math() {
            return None;
        }

        let clip = self.size().to_screen_int_rect(0, 0);

        let mut ctx_storage = ContextStorage::new();
        let mut blitter = RasterPipelineBlitter::new(paint, &mut ctx_storage, self)?;

        if paint.anti_alias {
            scan::path_aa::fill_path(path, paint.fill_type, &clip, &mut blitter)
        } else {
            scan::path::fill_path(path, paint.fill_type, &clip, &mut blitter)
        }
    }
}
