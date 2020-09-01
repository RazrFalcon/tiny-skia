// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Pixmap, Path, Color, BlendMode};

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


/// A paint used by a `Painter`.
#[allow(missing_copy_implementations)] // will became Clone-only later
#[derive(Clone, Debug)]
pub struct Paint {
    /// Paint color.
    ///
    /// Default: black
    pub color: Color,

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
}

impl Default for Paint {
    #[inline]
    fn default() -> Self {
        Paint {
            color: Color::BLACK,
            blend_mode: BlendMode::default(),
            fill_type: FillType::default(),
            anti_alias: false,
        }
    }
}

impl Paint {
    /// Sets a paint color.
    #[inline]
    pub fn set_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets a paint color.
    ///
    /// `paint.set_color(Color::from_rgba8(50, 127, 150, 200))` shorthand.
    #[inline]
    pub fn set_color_rgba8(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.set_color(Color::from_rgba8(r, g, b, a))
    }

    /// Sets a blending mode.
    #[inline]
    pub fn set_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Sets a fill type.
    #[inline]
    pub fn set_fill_type(mut self, fill_type: FillType) -> Self {
        self.fill_type = fill_type;
        self
    }

    /// Sets an anti-alias flag.
    #[inline]
    pub fn set_anti_alias(mut self, anti_alias: bool) -> Self {
        self.anti_alias = anti_alias;
        self
    }
}


/// A shapes painter.
pub trait Painter {
    /// Fills the entire pixmap with a specified color.
    ///
    /// This is essentially a memset, therefore it's very fast.
    fn fill(&mut self, color: Color);

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

    fn fill_path(&mut self, path: &Path, paint: &Paint) -> Option<()> {
        // This is sort of similar to SkDraw::drawPath

        // to_rect will fail when bounds' width/height is zero.
        // This is an intended behaviour since the only
        // reason for width/height to be zero is a horizontal/vertical line.
        // And in both cases there is nothing to fill.
        let path_bounds = path.bounds().to_rect()?;
        let path_int_bounds = path_bounds.round_out();

        // TODO: ignore ML paths
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
            scan::aa_path::fill_path(path, paint.fill_type, &clip, &mut blitter)
        } else {
            scan::path::fill_path(path, paint.fill_type, &clip, &mut blitter)
        }
    }
}
