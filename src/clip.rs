// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use tiny_skia_path::{IntRect, IntSize, ScreenIntRect, Transform};

use crate::{FillRule, LengthU32, Path};
use crate::{ALPHA_U8_OPAQUE, ALPHA_U8_TRANSPARENT};

use crate::alpha_runs::AlphaRun;
use crate::blitter::Blitter;
use crate::color::AlphaU8;
use crate::math::LENGTH_U32_ONE;
use crate::painter::DrawTiler;

use core::num::NonZeroU32;

/// A clipping mask.
///
/// Unlike Skia, we're using just a simple 8bit alpha mask.
/// It's way slower, but easier to implement.
#[derive(Clone, Debug)]
pub struct ClipMask {
    data: Vec<u8>,
    width: LengthU32,
    height: LengthU32,
}

impl Default for ClipMask {
    fn default() -> Self {
        ClipMask {
            data: Vec::new(),
            width: LENGTH_U32_ONE,
            height: LENGTH_U32_ONE,
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
        self.data.is_empty()
    }

    /// Returns mask size.
    pub(crate) fn size(&self) -> IntSize {
        IntSize::from_wh(self.width.get(), self.height.get()).unwrap()
    }

    pub(crate) fn as_submask<'a>(&'a self) -> SubClipMaskRef<'a> {
        SubClipMaskRef {
            size: self.size(),
            real_width: self.width,
            data: &self.data,
        }
    }

    pub(crate) fn submask<'a>(&'a self, rect: IntRect) -> Option<SubClipMaskRef<'a>> {
        let rect = self.size().to_int_rect(0, 0).intersect(&rect)?;
        let row_bytes = self.width.get() as usize;
        let offset = rect.top() as usize * row_bytes + rect.left() as usize;

        Some(SubClipMaskRef {
            size: rect.size(),
            real_width: self.width,
            data: &self.data[offset..],
        })
    }

    pub(crate) fn as_submask_mut<'a>(&'a mut self) -> SubClipMaskMut<'a> {
        SubClipMaskMut {
            size: self.size(),
            real_width: self.width,
            data: &mut self.data,
        }
    }

    pub(crate) fn submask_mut<'a>(&'a mut self, rect: IntRect) -> Option<SubClipMaskMut<'a>> {
        let rect = self.size().to_int_rect(0, 0).intersect(&rect)?;
        let row_bytes = self.width.get() as usize;
        let offset = rect.top() as usize * row_bytes + rect.left() as usize;

        Some(SubClipMaskMut {
            size: rect.size(),
            real_width: self.width,
            data: &mut self.data[offset..],
        })
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

        self.width = width;
        self.height = height;

        // Reuse the existing allocation.
        self.data.clear();
        self.data.resize((width.get() * height.get()) as usize, 0);

        if let Some(tiler) = DrawTiler::new(width.get(), height.get()) {
            let mut path = path.clone(); // TODO: avoid cloning

            for tile in tiler {
                let ts = Transform::from_translate(-(tile.x() as f32), -(tile.y() as f32));
                path = path.transform(ts)?;

                let submax = self.submask_mut(tile.to_int_rect())?;

                // We're ignoring "errors" here, because `fill_path` will return `None`
                // when rendering a tile that doesn't have a path on it.
                // Which is not an error in this case.
                let clip_rect = tile.size().to_screen_int_rect(0, 0);
                if anti_alias {
                    let mut builder = ClipBuilderAA(submax);
                    let _ =
                        crate::scan::path_aa::fill_path(&path, fill_rule, &clip_rect, &mut builder);
                } else {
                    let mut builder = ClipBuilder(submax);
                    let _ =
                        crate::scan::path::fill_path(&path, fill_rule, &clip_rect, &mut builder);
                }

                let ts = Transform::from_translate(tile.x() as f32, tile.y() as f32);
                path = path.transform(ts)?;
            }

            Some(())
        } else {
            let clip = ScreenIntRect::from_xywh_safe(0, 0, width, height);
            if anti_alias {
                let mut builder = ClipBuilderAA(self.as_submask_mut());
                crate::scan::path_aa::fill_path(path, fill_rule, &clip, &mut builder)
            } else {
                let mut builder = ClipBuilder(self.as_submask_mut());
                crate::scan::path::fill_path(path, fill_rule, &clip, &mut builder)
            }
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
        submask.set_path(
            self.width.get(),
            self.height.get(),
            path,
            fill_rule,
            anti_alias,
        )?;

        for (a, b) in self.data.iter_mut().zip(submask.data.iter()) {
            *a = crate::color::premultiply_u8(*a, *b);
        }

        Some(())
    }

    /// Clears the mask.
    ///
    /// Internal memory buffer is not deallocated.
    pub fn clear(&mut self) {
        // Clear the mask, but keep the allocation.
        self.data.clear();
    }
}

#[derive(Clone, Copy)]
pub struct SubClipMaskRef<'a> {
    pub data: &'a [u8],
    pub size: IntSize,
    pub real_width: LengthU32,
}

impl<'a> SubClipMaskRef<'a> {
    pub(crate) fn clip_mask_ctx(&self) -> crate::pipeline::ClipMaskCtx<'a> {
        crate::pipeline::ClipMaskCtx {
            data: &self.data,
            stride: self.real_width,
        }
    }
}

// Similar to SubPixmapMut.
pub struct SubClipMaskMut<'a> {
    pub data: &'a mut [u8],
    pub size: IntSize,
    pub real_width: LengthU32,
}

struct ClipBuilder<'a>(SubClipMaskMut<'a>);

impl Blitter for ClipBuilder<'_> {
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        let offset = (y * self.0.real_width.get() + x) as usize;
        for i in 0..width.get() as usize {
            self.0.data[offset + i] = 255;
        }
    }
}

struct ClipBuilderAA<'a>(SubClipMaskMut<'a>);

impl Blitter for ClipBuilderAA<'_> {
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        let offset = (y * self.0.real_width.get() + x) as usize;
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
                    let offset = (y * self.0.real_width.get() + x) as usize;
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
