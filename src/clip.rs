// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use crate::{Path, LengthU32, FillRule};
use crate::{ALPHA_U8_OPAQUE, ALPHA_U8_TRANSPARENT};

use crate::alpha_runs::AlphaRun;
use crate::blitter::Blitter;
use crate::color::AlphaU8;
use crate::geom::ScreenIntRect;
use crate::math::LENGTH_U32_ONE;
use core::num::NonZeroU32;

#[derive(Clone, Debug)]
pub struct ClipMaskData {
    pub data: Vec<u8>,
    pub width: LengthU32,
    pub height: LengthU32,
}

impl ClipMaskData {
    pub(crate) fn clip_mask_ctx(&self) -> crate::pipeline::ClipMaskCtx {
        crate::pipeline::ClipMaskCtx {
            data: &self.data,
            stride: self.width,
        }
    }
}


/// A clipping mask.
///
/// Unlike Skia, we're using just a simple 8bit alpha mask.
/// It's way slower, but times easier to implement.
#[derive(Clone, Debug)]
pub struct ClipMask {
    pub(crate) mask: ClipMaskData,
}

impl Default for ClipMask {
    fn default() -> Self {
        ClipMask {
            mask: ClipMaskData {
                data: Vec::new(),
                width: LENGTH_U32_ONE,
                height: LENGTH_U32_ONE,
            }
        }
    }
}

impl ClipMask {
    /// Creates a new, empty mask.
    pub fn new() -> Self {
        ClipMask::default()
    }

    /// Checks that mask is empty.
    pub fn is_empty(&self) -> bool {
        self.mask.data.is_empty()
    }

    /// Sets the current clipping path.
    ///
    /// Not additive. Overwrites the previous data.
    ///
    /// Path must be transformed beforehand.
    pub fn set_path(
        &mut self,
        width: u32,
        height: u32,
        path: &Path,
        fill_rule: FillRule,
        anti_alias: bool,
    ) -> Option<()> {
        let width = NonZeroU32::new(width)?;
        let height = NonZeroU32::new(height)?;

        self.mask.width = width;
        self.mask.height = height;

        // Reuse the existing allocation.
        self.mask.data.clear();
        self.mask.data.resize((width.get() * height.get()) as usize, 0);

        let clip = ScreenIntRect::from_xywh_safe(0, 0, width, height);

        if anti_alias {
            let mut builder = ClipBuilderAA(&mut self.mask);
            crate::scan::path_aa::fill_path(path, fill_rule, &clip, &mut builder)
        } else {
            let mut builder = ClipBuilder(&mut self.mask);
            crate::scan::path::fill_path(path, fill_rule, &clip, &mut builder)
        }
    }

    /// Intersects the provided path with the current clipping path.
    ///
    /// Path must be transformed beforehand.
    pub fn intersect_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        anti_alias: bool,
    ) -> Option<()> {
        let mut submask = ClipMask::new();
        submask.set_path(self.mask.width.get(), self.mask.height.get(), path, fill_rule, anti_alias)?;

        for (a, b) in self.mask.data.iter_mut().zip(submask.mask.data.iter()) {
            *a = (((u16::from(*a)) * u16::from(*b)) >> 8) as u8;
        }

        Some(())
    }

    /// Clears the mask.
    ///
    /// Internal memory buffer is not deallocated.
    pub fn clear(&mut self) {
        // Clear the mask, but keep the allocation.
        self.mask.data.clear();
    }
}


struct ClipBuilder<'a>(&'a mut ClipMaskData);

impl Blitter for ClipBuilder<'_> {
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        let offset = (y * self.0.width.get() + x) as usize;
        for i in 0..width.get() as usize {
            self.0.data[offset + i] = 255;
        }
    }
}


struct ClipBuilderAA<'a>(&'a mut ClipMaskData);

impl Blitter for ClipBuilderAA<'_> {
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        let offset = (y * self.0.width.get() + x) as usize;
        for i in 0..width.get() as usize {
            self.0.data[offset + i] = 255;
        }
    }

    fn blit_anti_h(&mut self, mut x: u32, y: u32, aa: &mut [AlphaU8], runs: &mut [AlphaRun]) {
        let mut aa_offset = 0;
        let mut run_offset = 0;
        let mut run_opt = runs[0];
        while let Some(run) = run_opt {
            let width = LengthU32::from(run);

            match aa[aa_offset] {
                ALPHA_U8_TRANSPARENT => {}
                ALPHA_U8_OPAQUE => {
                    self.blit_h(x, y, width);
                }
                alpha => {
                    let offset = (y * self.0.width.get() + x) as usize;
                    for i in 0..width.get() as usize {
                        self.0.data[offset + i] = alpha;
                    }
                }
            }

            x += width.get();
            run_offset += usize::from(run.get());
            aa_offset += usize::from(run.get());
            run_opt = runs[run_offset];
        }
    }
}
