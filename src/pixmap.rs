// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::convert::TryFrom;
use std::num::NonZeroUsize;

use crate::{Color, IntRect};

use crate::color::PremultipliedColorU8;
use crate::geom::{IntSize, ScreenIntRect};

#[cfg(feature = "png-format")]
use crate::color::{premultiply_u8, ALPHA_U8_OPAQUE};

/// Number of bytes per pixel.
pub const BYTES_PER_PIXEL: usize = 4;


/// A container that owns premultiplied RGBA pixels.
///
/// The data is not aligned, therefore width == stride.
#[derive(Clone, PartialEq)]
pub struct Pixmap {
    data: Vec<u8>,
    size: IntSize,
}

impl Pixmap {
    /// Allocates a new pixmap.
    ///
    /// A pixmap is filled with transparent black by default, aka (0, 0, 0, 0).
    ///
    /// Zero size in an error.
    ///
    /// Pixmap's width is limited by i32::MAX/4.
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let data_len = data_len_for_size(size)?;

        // We cannot check that allocation was successful yet.
        // We have to wait for https://github.com/rust-lang/rust/issues/48043

        Some(Pixmap {
            data: vec![0; data_len],
            size,
        })
    }

    #[cfg(feature = "png-format")]
    pub(crate) fn from_vec(data: Vec<u8>, size: IntSize) -> Option<Self> {
        let data_len = data_len_for_size(size)?;
        if data.len() != data_len {
            return None;
        }

        Some(Pixmap {
            data,
            size,
        })
    }

    /// Decodes a PNG data into a `Pixmap`.
    ///
    /// Only 8-bit images are supported.
    /// Index PNGs are not supported.
    #[cfg(feature = "png-format")]
    pub fn decode_png(data: &[u8]) -> Result<Self, png::DecodingError> {
        let decoder = png::Decoder::new(data);
        let (info, mut reader) = decoder.read_info()?;

        if info.bit_depth != png::BitDepth::Eight {
            return Err(png::DecodingError::from("unsupported bit depth".to_string()));
        }

        let size = IntSize::from_wh(info.width, info.height)
            .ok_or_else(|| png::DecodingError::from("invalid image size".to_string()))?;
        let data_len = data_len_for_size(size)
            .ok_or_else(|| png::DecodingError::from("image is too big".to_string()))?;

        let mut img_data = vec![0; info.buffer_size()];
        reader.next_frame(&mut img_data)?;

        img_data = match info.color_type {
            png::ColorType::RGB => {
                let mut rgba_data = Vec::with_capacity(data_len);
                for rgb in img_data.chunks(3) {
                    rgba_data.push(rgb[0]);
                    rgba_data.push(rgb[1]);
                    rgba_data.push(rgb[2]);
                    rgba_data.push(ALPHA_U8_OPAQUE);
                }

                rgba_data
            }
            png::ColorType::RGBA => {
                img_data
            }
            png::ColorType::Grayscale => {
                let mut rgba_data = Vec::with_capacity(data_len);
                for gray in img_data {
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(ALPHA_U8_OPAQUE);
                }

                rgba_data
            }
            png::ColorType::GrayscaleAlpha => {
                let mut rgba_data = Vec::with_capacity(data_len);
                for slice in img_data.chunks(2) {
                    let gray = slice[0];
                    let alpha = slice[1];
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(gray);
                    rgba_data.push(alpha);
                }

                rgba_data
            }
            png::ColorType::Indexed => {
                return Err(png::DecodingError::from("indexed PNG is not supported".to_string()));
            }
        };

        // Premultiply alpha.
        //
        // We cannon use RasterPipeline here, which is faster,
        // because it produces slightly different results.
        // Seems like Skia does the same.
        //
        // Also, in our tests unsafe version (no bound checking)
        // had roughly the same performance. So we keep the safe one.
        for pixel in img_data.as_mut_slice().chunks_mut(BYTES_PER_PIXEL) {
            let a = pixel[3];
            pixel[0] = premultiply_u8(pixel[0], a);
            pixel[1] = premultiply_u8(pixel[1], a);
            pixel[2] = premultiply_u8(pixel[2], a);
        }

        Pixmap::from_vec(img_data, size)
            .ok_or_else(|| png::DecodingError::from("failed to create a pixmap".to_string()))
    }

    /// Loads a PNG file into a `Pixmap`.
    ///
    /// Only 8-bit images are supported.
    /// Index PNGs are not supported.
    #[cfg(feature = "png-format")]
    pub fn load_png<P: AsRef<std::path::Path>>(path: P) -> Result<Self, png::DecodingError> {
        // `png::Decoder` is generic over input, which means that it will instance
        // two copies: one for `&[]` and one for `File`. Which will simply bloat the code.
        // Therefore we're using only one type for input.
        let data = std::fs::read(path)?;
        Self::decode_png(&data)
    }

    /// Encodes pixmap into a PNG data.
    #[cfg(feature = "png-format")]
    pub fn encode_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        // Skia uses skcms here, which is somewhat similar to RasterPipeline.

        // Sadly, we have to copy the pixmap here.
        // Not sure how to avoid this.
        let mut tmp_pixmap = self.clone();

        // Demultiply alpha.
        //
        // RasterPipeline is 15% faster here, but produces slightly different results
        // due to rounding. So we stick with this method for now.
        for pixel in tmp_pixmap.as_mut().pixels_mut() {
            let c = pixel.demultiply();
            *pixel = PremultipliedColorU8::from_rgba_unchecked(
                c.red(), c.green(), c.blue(), c.alpha());
        }

        let mut data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut data, self.width(), self.height());
            encoder.set_color(png::ColorType::RGBA);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(&tmp_pixmap.data)?;
        }

        Ok(data)
    }

    /// Saves pixmap as a PNG file.
    #[cfg(feature = "png-format")]
    pub fn save_png<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), png::EncodingError> {
        let data = self.encode_png()?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Returns a container that references Pixmap's data.
    pub fn as_ref(&self) -> PixmapRef {
        PixmapRef {
            data: &self.data,
            size: self.size,
        }
    }

    /// Returns a container that references Pixmap's data.
    pub fn as_mut(&mut self) -> PixmapMut {
        PixmapMut {
            data: &mut self.data,
            size: self.size,
        }
    }

    /// Returns pixmap's width.
    pub fn width(&self) -> u32 {
        self.size.width()
    }

    /// Returns pixmap's height.
    pub fn height(&self) -> u32 {
        self.size.height()
    }

    /// Returns pixmap's size.
    #[allow(dead_code)]
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    // /// Returns pixmap's rect.
    // pub(crate) fn rect(&self) -> ScreenIntRect {
    //     self.size.to_screen_int_rect(0, 0)
    // }

    /// Fills the entire pixmap with a specified color.
    pub fn fill(&mut self, color: Color) {
        let c = color.premultiply().to_color_u8();
        for p in self.as_mut().pixels_mut() {
            *p = c;
        }
    }

    /// Returns the internal data.
    ///
    /// Bytes are ordered as RGBA.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Returns the mutable internal data.
    ///
    /// Bytes are ordered as RGBA.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    /// Returns a pixel color.
    ///
    /// Returns `None` when position is out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<PremultipliedColorU8> {
        let idx = self.width().checked_mul(y)?.checked_add(x)?;
        self.pixels().get(idx as usize).cloned()
    }

    /// Returns a mutable slice of pixels.
    pub fn pixels_mut(&mut self) -> &mut [PremultipliedColorU8] {
        bytemuck::cast_slice_mut(self.data_mut())
    }

    /// Returns a slice of pixels.
    pub fn pixels(&self) -> &[PremultipliedColorU8] {
        bytemuck::cast_slice(self.data())
    }

    /// Consumes the internal data.
    ///
    /// Bytes are ordered as RGBA.
    pub fn take(self) -> Vec<u8> {
        self.data
    }

    /// Returns a copy of the pixmap that intersects the `rect`.
    ///
    /// Returns `None` when `Pixmap`'s rect doesn't contain `rect`.
    pub fn clone_rect(&self, rect: IntRect) -> Option<Pixmap> {
        self.as_ref().clone_rect(rect)
    }
}

impl std::fmt::Debug for Pixmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pixmap")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}


/// A container that references premultiplied RGBA pixels.
///
/// Can be created from `Pixmap` or from a user provided data.
///
/// The data is not aligned, therefore width == stride.
#[derive(Clone, Copy, PartialEq)]
pub struct PixmapRef<'a> {
    data: &'a [u8],
    size: IntSize,
}

impl<'a> PixmapRef<'a> {
    /// Creates a new `PixmapRef` from bytes.
    ///
    /// The size must be at least `size.width() * size.height() * BYTES_PER_PIXEL`.
    /// Zero size in an error. Width is limited by i32::MAX/4.
    ///
    /// The `data` is assumed to have premultiplied RGBA pixels (byteorder: ABGR).
    pub fn from_bytes(data: &'a [u8], width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let data_len = data_len_for_size(size)?;
        if data.len() < data_len {
            return None;
        }

        Some(PixmapRef {
            data,
            size,
        })
    }

    /// Creates a new `Pixmap` from the current data.
    ///
    /// Clones the underlying data.
    pub fn to_owned(&self) -> Pixmap {
        Pixmap {
            data: self.data.to_vec(),
            size: self.size,
        }
    }

    /// Returns pixmap's width.
    pub fn width(&self) -> u32 {
        self.size.width()
    }

    /// Returns pixmap's height.
    pub fn height(&self) -> u32 {
        self.size.height()
    }

    /// Returns pixmap's size.
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    /// Returns pixmap's rect.
    pub(crate) fn rect(&self) -> ScreenIntRect {
        self.size.to_screen_int_rect(0, 0)
    }

    /// Returns the internal data.
    ///
    /// Bytes are ordered as RGBA.
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    /// Returns a pixel color.
    ///
    /// Returns `None` when position is out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<PremultipliedColorU8> {
        let idx = self.width().checked_mul(y)?.checked_add(x)?;
        self.pixels().get(idx as usize).cloned()
    }

    /// Returns a slice of pixels.
    pub fn pixels(&self) -> &'a [PremultipliedColorU8] {
        bytemuck::cast_slice(self.data())
    }

    // TODO: add rows() iterator

    /// Returns a copy of the pixmap that intersects the `rect`.
    ///
    /// Returns `None` when `Pixmap`'s rect doesn't contain `rect`.
    pub fn clone_rect(&self, rect: IntRect) -> Option<Pixmap> {
        // TODO: to ScreenIntRect?

        let rect = self.rect().to_int_rect().intersect(&rect)?;
        let mut new = Pixmap::new(rect.width(), rect.height())?;
        {
            let old_pixels = self.pixels();
            let mut new_mut = new.as_mut();
            let new_pixels = new_mut.pixels_mut();

            // TODO: optimize
            for y in 0..rect.height() {
                for x in 0..rect.width() {
                    let old_idx = (y + rect.y() as u32) * self.width() + (x + rect.x() as u32);
                    let new_idx = y * rect.width() + x;
                    new_pixels[new_idx as usize] = old_pixels[old_idx as usize];
                }
            }
        }

        Some(new)
    }
}

impl std::fmt::Debug for PixmapRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixmapRef")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}


/// A container that references mutable premultiplied RGBA pixels.
///
/// Can be created from `Pixmap` or from a user provided data.
///
/// The data is not aligned, therefore width == stride.
#[derive(PartialEq)]
pub struct PixmapMut<'a> {
    data: &'a mut [u8],
    size: IntSize,
}

impl<'a> PixmapMut<'a> {
    /// Creates a new `PixmapMut` from bytes.
    ///
    /// The size must be at least `size.width() * size.height() * BYTES_PER_PIXEL`.
    /// Zero size in an error. Width is limited by i32::MAX/4.
    ///
    /// The `data` is assumed to have premultiplied RGBA pixels (byteorder: ABGR).
    pub fn from_bytes(data: &'a mut [u8], width: u32, height: u32) -> Option<Self> {
        let size = IntSize::from_wh(width, height)?;
        let data_len = data_len_for_size(size)?;
        if data.len() < data_len {
            return None;
        }

        Some(PixmapMut {
            data,
            size,
        })
    }

    /// Creates a new `Pixmap` from the current data.
    ///
    /// Clones the underlying data.
    pub fn to_owned(&self) -> Pixmap {
        Pixmap {
            data: self.data.to_vec(),
            size: self.size,
        }
    }

    /// Returns pixmap's width.
    pub fn width(&self) -> u32 {
        self.size.width()
    }

    /// Returns pixmap's height.
    pub fn height(&self) -> u32 {
        self.size.height()
    }

    /// Returns pixmap's size.
    pub(crate) fn size(&self) -> IntSize {
        self.size
    }

    /// Returns pixmap's rect.
    pub(crate) fn rect(&self) -> ScreenIntRect {
        self.size.to_screen_int_rect(0, 0)
    }

    /// Returns the mutable internal data.
    ///
    /// Bytes are ordered as RGBA.
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data
    }

    /// Returns a mutable slice of pixels.
    pub fn pixels_mut(&mut self) -> &mut [PremultipliedColorU8] {
        bytemuck::cast_slice_mut(self.data_mut())
    }
}

impl std::fmt::Debug for PixmapMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixmapMut")
            .field("data", &"...")
            .field("width", &self.size.width())
            .field("height", &self.size.height())
            .finish()
    }
}


/// Returns minimum bytes per row as usize.
///
/// Pixmap's maximum value for row bytes must fit in 31 bits.
fn min_row_bytes(size: IntSize) -> Option<NonZeroUsize> {
    let w = i32::try_from(size.width()).ok()?;
    let w = w.checked_mul(BYTES_PER_PIXEL as i32)?;
    NonZeroUsize::new(w as usize)
}

/// Returns storage size required by pixel array.
fn compute_data_len(size: IntSize, row_bytes: usize) -> Option<usize> {
    let h = size.height().checked_sub(1)?;
    let h = (h as usize).checked_mul(row_bytes)?;

    let w = (size.width() as usize).checked_mul(BYTES_PER_PIXEL)?;

    h.checked_add(w)
}

fn data_len_for_size(size: IntSize) -> Option<usize> {
    let row_bytes = min_row_bytes(size)?;
    compute_data_len(size, row_bytes.get())
}
