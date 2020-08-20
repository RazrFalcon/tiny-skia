// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::convert::TryFrom;
use std::num::NonZeroUsize;

use crate::{IntSize, LengthU32};

use crate::color::PremultipliedColorU8;

#[cfg(feature = "png-format")]
use crate::color::{premultiply_u8, ALPHA_U8_OPAQUE};

const BYTES_PER_PIXEL: usize = 4;


/// A container of premultiplied RGBA pixels.
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

    /// Returns pixmap's width.
    #[inline]
    pub fn width(&self) -> LengthU32 {
        self.size.width()
    }

    /// Returns pixmap's height.
    #[inline]
    pub fn height(&self) -> LengthU32 {
        self.size.height()
    }

    /// Returns pixmap's size.
    #[inline]
    pub fn size(&self) -> IntSize {
        self.size
    }

    /// Returns an internal data.
    ///
    /// Bytes are ordered as RGBA.
    #[inline]
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Returns a pixel color.
    ///
    /// Returns `None` when position is out of bounds.
    #[inline]
    pub fn pixel(&self, x: u32, y: u32) -> Option<PremultipliedColorU8> {
        let idx = self.width().get().checked_mul(y)?.checked_add(x)?;
        self.pixels().get(idx as usize).cloned()
    }

    /// Returns a slice of pixels.
    #[inline]
    pub fn pixels(&self) -> &[PremultipliedColorU8] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const PremultipliedColorU8,
                self.data.len() / BYTES_PER_PIXEL,
            )
        }
    }

    /// Returns a mutable slice of pixels.
    #[inline]
    pub(crate) fn pixels_mut(&mut self) -> &mut [PremultipliedColorU8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_ptr() as *mut PremultipliedColorU8,
                self.data.len() / BYTES_PER_PIXEL,
            )
        }
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
        let file = std::fs::File::open(path)?;
        let data = unsafe { &memmap2::MmapOptions::new().map(&file)? };
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
        for pixel in tmp_pixmap.pixels_mut() {
            let c = pixel.demultiply();
            *pixel = PremultipliedColorU8::from_rgba_unchecked(c.red(), c.green(), c.blue(), c.alpha());
        }

        let mut data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut data, self.width().get(), self.height().get());
            encoder.set_color(png::ColorType::RGBA);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(tmp_pixmap.data())?;
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


/// Returns minimum bytes per row as usize.
///
/// Pixmap's maximum value for row bytes must fit in 31 bits.
#[inline]
fn min_row_bytes(size: IntSize) -> Option<NonZeroUsize> {
    let w = i32::try_from(size.width().get()).ok()?;
    let w = w.checked_mul(BYTES_PER_PIXEL as i32)?;
    NonZeroUsize::new(w as usize)
}

/// Returns storage required by pixel array.
fn compute_data_len(size: IntSize, row_bytes: usize) -> Option<usize> {
    let h = size.height().get().checked_sub(1)?;
    let h = (h as usize).checked_mul(row_bytes)?;

    let w = (size.width().get() as usize).checked_mul(BYTES_PER_PIXEL)?;

    h.checked_add(w)
}

#[inline]
fn data_len_for_size(size: IntSize) -> Option<usize> {
    let row_bytes = min_row_bytes(size)?;
    compute_data_len(size, row_bytes.get())
}
