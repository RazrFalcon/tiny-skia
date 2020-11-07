// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Unlike Skia, we're using just a simple 1bit alpha mask for clipping.

use crate::{Path, LengthU32, FillRule};
use crate::{ALPHA_U8_OPAQUE, ALPHA_U8_TRANSPARENT};

use crate::screen_int_rect::ScreenIntRect;
use crate::blitter::Blitter;
use crate::math::LENGTH_U32_ONE;
use crate::color::AlphaU8;
use crate::alpha_runs::AlphaRun;

#[derive(Clone)]
pub struct ClipMask {
    pub data: Vec<u8>,
    pub width: LengthU32,
}

#[derive(Clone)]
pub struct Clip {
    mask: ClipMask,
}

impl Clip {
    pub fn new() -> Self {
        Clip {
            mask: ClipMask {
                data: Vec::new(),
                width: LENGTH_U32_ONE,
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mask.data.is_empty()
    }

    pub fn as_ref(&self) -> Option<&ClipMask> {
        if self.is_empty() {
            None
        } else {
            Some(&self.mask)
        }
    }

    pub fn set_path(
        &mut self,
        path: &Path,
        clip: ScreenIntRect,
        fill_type: FillRule,
        anti_alias: bool,
    ) -> Option<()> {
        self.mask.width = clip.width_safe();

        // Reuse the existing allocation.
        self.mask.data.clear();
        self.mask.data.resize((clip.width() * clip.height()) as usize, 0);

        if anti_alias {
            let mut builder = ClipBuilderAA(&mut self.mask);
            crate::scan::path_aa::fill_path(path, fill_type, &clip, &mut builder)
        } else {
            let mut builder = ClipBuilder(&mut self.mask);
            crate::scan::path::fill_path(path, fill_type, &clip, &mut builder)
        }
    }

    pub fn clear(&mut self) {
        // Clear the mask, but keep the allocation.
        self.mask.data.clear();
    }
}


struct ClipBuilder<'a>(&'a mut ClipMask);

impl Blitter for ClipBuilder<'_> {
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        let offset = (y * self.0.width.get() + x) as usize;
        for i in 0..width.get() as usize {
            self.0.data[offset + i] = 255;
        }
    }
}


struct ClipBuilderAA<'a>(&'a mut ClipMask);

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
