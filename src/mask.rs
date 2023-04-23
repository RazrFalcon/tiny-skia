// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use tiny_skia_path::NoStdFloat;

use alloc::vec;
use alloc::vec::Vec;

use tiny_skia_path::{IntRect, IntSize, Path, Scalar, Transform};

use crate::painter::DrawTiler;
use crate::pipeline::RasterPipelineBlitter;
use crate::pixmap::SubPixmapMut;
use crate::scan;
use crate::{FillRule, PixmapRef};

/// A mask type.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MaskType {
    /// Transfers only the Alpha channel from `Pixmap` to `Mask`.
    Alpha,
    /// Transfers RGB channels as perceived luminosity from `Pixmap` to `Mask`.
    ///
    /// Formula: `Y = 0.2126 * R + 0.7152 * G + 0.0722 * B`
    Luminosity,
}

/// A mask.
///
/// During drawing over `Pixmap`, mask's black (0) "pixels" would block rendering
/// and white (255) will allow it.
/// Anything in between is used for gradual masking and anti-aliasing.
///
/// Unlike Skia, we're using just a simple 8bit alpha mask.
/// It's way slower, but easier to implement.
#[derive(Clone, PartialEq)]
pub struct Mask {
    data: Vec<u8>,
    size: IntSize,
}

impl Mask {
    /// Creates a new mask by taking ownership over a mask buffer.
    ///
    /// The size needs to match the data provided.
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        Some(Mask {
            data: vec![0; width as usize * height as usize],
            size,
        })
    }

    /// Creates a new mask from a `PixmapRef`.
    pub fn from_pixmap(pixmap: PixmapRef, mask_type: MaskType) -> Self {
        let data_len = pixmap.width() as usize * pixmap.height() as usize;
        let mut mask = Mask {
            data: vec![0; data_len],
            size: pixmap.size(),
        };

        // TODO: optimize
        match mask_type {
            MaskType::Alpha => {
                for (p, a) in pixmap.pixels().iter().zip(mask.data.as_mut_slice()) {
                    *a = p.alpha();
                }
            }
            MaskType::Luminosity => {
                for (p, a) in pixmap.pixels().iter().zip(mask.data.as_mut_slice()) {
                    let p = p.to_premultiplied_color().demultiply();
                    let luma = p.red() * 0.2126 + p.green() * 0.7152 + p.blue() * 0.0722;
                    *a = ((luma * p.alpha()) * 255.0).clamp(0.0, 255.0).ceil() as u8;
                }
            }
        }

        mask
    }

    /// Creates a new mask by taking ownership over a mask buffer.
    ///
    /// The size needs to match the data provided.
    pub fn from_vec(data: Vec<u8>, size: IntSize) -> Option<Self> {
        let data_len = size.width() as usize * size.height() as usize;
        if data.len() != data_len {
            return None;
        }

        Some(Mask { data, size })
    }

    /// Returns mask's width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.size.width()
    }

    /// Returns mask's height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.size.height()
    }

    /// Returns mask's size.
    #[allow(dead_code)]
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    /// Returns the internal data.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Returns the mutable internal data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    pub(crate) fn as_submask<'a>(&'a self) -> SubMaskRef<'a> {
        SubMaskRef {
            size: self.size,
            real_width: self.size.width(),
            data: &self.data,
        }
    }

    pub(crate) fn submask<'a>(&'a self, rect: IntRect) -> Option<SubMaskRef<'a>> {
        let rect = self.size.to_int_rect(0, 0).intersect(&rect)?;
        let row_bytes = self.width() as usize;
        let offset = rect.top() as usize * row_bytes + rect.left() as usize;

        Some(SubMaskRef {
            size: rect.size(),
            real_width: self.size.width(),
            data: &self.data[offset..],
        })
    }

    pub(crate) fn as_subpixmap<'a>(&'a mut self) -> SubPixmapMut<'a> {
        SubPixmapMut {
            size: self.size,
            real_width: self.size.width() as usize,
            data: &mut self.data,
        }
    }

    pub(crate) fn subpixmap<'a>(&'a mut self, rect: IntRect) -> Option<SubPixmapMut<'a>> {
        let rect = self.size.to_int_rect(0, 0).intersect(&rect)?;
        let row_bytes = self.width() as usize;
        let offset = rect.top() as usize * row_bytes + rect.left() as usize;

        Some(SubPixmapMut {
            size: rect.size(),
            real_width: self.size.width() as usize,
            data: &mut self.data[offset..],
        })
    }

    /// Encodes mask into a PNG data.
    #[cfg(feature = "png-format")]
    pub fn encode_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        let mut data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut data, self.width(), self.height());
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(&self.data)?;
        }

        Ok(data)
    }

    /// Saves mask as a PNG file.
    #[cfg(feature = "png-format")]
    pub fn save_png<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), png::EncodingError> {
        let data = self.encode_png()?;
        std::fs::write(path, data)?;
        Ok(())
    }

    // Almost a direct copy of PixmapMut::fill_path
    /// Draws a filled path onto the mask.
    ///
    /// In terms of RGB (no alpha) image, draws a white path on top of black mask.
    ///
    /// Doesn't reset the existing mask content and draws the path on top of existing data.
    ///
    /// If the above behavior is undesired, [`clear()`] should be called first.
    ///
    /// This method is intended to be used for simple cases. For more complex masks
    /// prefer [`from_pixmap()`].
    pub fn fill_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        anti_alias: bool,
        transform: Transform,
    ) {
        if transform.is_identity() {
            // This is sort of similar to SkDraw::drawPath

            // Skip empty paths and horizontal/vertical lines.
            let path_bounds = path.bounds();
            if path_bounds.width().is_nearly_zero() || path_bounds.height().is_nearly_zero() {
                log::warn!("empty paths and horizontal/vertical lines cannot be filled");
                return;
            }

            if crate::painter::is_too_big_for_math(path) {
                log::warn!("path coordinates are too big");
                return;
            }

            // TODO: ignore paths outside the pixmap

            if let Some(tiler) = DrawTiler::new(self.width(), self.height()) {
                let mut path = path.clone(); // TODO: avoid cloning

                for tile in tiler {
                    let ts = Transform::from_translate(-(tile.x() as f32), -(tile.y() as f32));
                    path = match path.transform(ts) {
                        Some(v) => v,
                        None => {
                            log::warn!("path transformation failed");
                            return;
                        }
                    };

                    let clip_rect = tile.size().to_screen_int_rect(0, 0);
                    let mut subpix = match self.subpixmap(tile.to_int_rect()) {
                        Some(v) => v,
                        None => continue, // technically unreachable
                    };

                    let mut blitter = match RasterPipelineBlitter::new_mask(&mut subpix) {
                        Some(v) => v,
                        None => continue, // nothing to do, all good
                    };

                    // We're ignoring "errors" here, because `fill_path` will return `None`
                    // when rendering a tile that doesn't have a path on it.
                    // Which is not an error in this case.
                    if anti_alias {
                        scan::path_aa::fill_path(&path, fill_rule, &clip_rect, &mut blitter);
                    } else {
                        scan::path::fill_path(&path, fill_rule, &clip_rect, &mut blitter);
                    }

                    let ts = Transform::from_translate(tile.x() as f32, tile.y() as f32);
                    path = match path.transform(ts) {
                        Some(v) => v,
                        None => return, // technically unreachable
                    };
                }
            } else {
                let clip_rect = self.size().to_screen_int_rect(0, 0);
                let mut subpix = self.as_subpixmap();
                let mut blitter = match RasterPipelineBlitter::new_mask(&mut subpix) {
                    Some(v) => v,
                    None => return, // nothing to do, all good
                };

                if anti_alias {
                    scan::path_aa::fill_path(path, fill_rule, &clip_rect, &mut blitter);
                } else {
                    scan::path::fill_path(path, fill_rule, &clip_rect, &mut blitter);
                }
            }
        } else {
            let path = match path.clone().transform(transform) {
                Some(v) => v,
                None => {
                    log::warn!("path transformation failed");
                    return;
                }
            };

            self.fill_path(&path, fill_rule, anti_alias, Transform::identity());
        }
    }

    /// Intersects the provided path with the current clipping path.
    ///
    /// A temporary mask with the same size as the current one will be created.
    pub fn intersect_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        anti_alias: bool,
        transform: Transform,
    ) {
        let mut submask = Mask::new(self.width(), self.height()).unwrap();
        submask.fill_path(path, fill_rule, anti_alias, transform);

        for (a, b) in self.data.iter_mut().zip(submask.data.iter()) {
            *a = crate::color::premultiply_u8(*a, *b);
        }
    }

    /// Clears the mask.
    ///
    /// Zero-fills the internal data buffer.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }
}

impl core::fmt::Debug for Mask {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mask")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct SubMaskRef<'a> {
    pub data: &'a [u8],
    pub size: IntSize,
    pub real_width: u32,
}

impl<'a> SubMaskRef<'a> {
    pub(crate) fn mask_ctx(&self) -> crate::pipeline::MaskCtx<'a> {
        crate::pipeline::MaskCtx {
            data: &self.data,
            real_width: self.real_width,
        }
    }
}
