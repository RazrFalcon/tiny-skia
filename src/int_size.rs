// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{LengthU32, IntRect};

use crate::screen_int_rect::ScreenIntRect;

/// An integer size.
///
/// # Guarantees
///
/// - Width and height are positive and non-zero.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct IntSize {
    width: LengthU32,
    height: LengthU32,
}

impl IntSize {
    /// Creates a new `IntSize` from width and height.
    #[inline]
    pub fn from_wh(width: u32, height: u32) -> Option<Self> {
        Some(IntSize {
            width: LengthU32::new(width)?,
            height: LengthU32::new(height)?,
        })
    }

    /// Returns width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Converts the current size into a `IntRect` at a provided position.
    #[inline]
    pub fn to_int_rect(&self, x: i32, y: i32) -> IntRect {
        IntRect::from_xywh(x, y, self.width.get(), self.height.get()).unwrap()
    }

    /// Converts the current size into a `IntRect` at a provided position.
    #[inline]
    pub(crate) fn to_screen_int_rect(&self, x: u32, y: u32) -> ScreenIntRect {
        ScreenIntRect::from_xywh_safe(x, y, self.width, self.height)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(IntSize::from_wh(0, 0), None);
        assert_eq!(IntSize::from_wh(1, 0), None);
        assert_eq!(IntSize::from_wh(0, 1), None);

        let size = IntSize::from_wh(3, 4).unwrap();
        assert_eq!(size.to_int_rect(1, 2), IntRect::from_xywh(1, 2, 3, 4).unwrap());
        assert_eq!(size.to_screen_int_rect(1, 2), ScreenIntRect::from_xywh(1, 2, 3, 4).unwrap());
    }
}
