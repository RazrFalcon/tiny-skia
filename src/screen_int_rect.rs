// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::convert::TryFrom;

use crate::{LengthU32, IntSize, IntRect, Rect};

/// A screen `IntRect`.
///
/// # Guarantees
///
/// - X and Y are in 0..=i32::MAX range.
/// - Width and height are in 1..=i32::MAX range.
/// - x+width and y+height does not overflow.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ScreenIntRect {
    x: u32,
    y: u32,
    width: LengthU32,
    height: LengthU32,
}

impl ScreenIntRect {
    /// Creates a new `ScreenIntRect`.
    #[inline]
    pub fn from_xywh(x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
        i32::try_from(x).ok()?;
        i32::try_from(y).ok()?;
        i32::try_from(width).ok()?;
        i32::try_from(height).ok()?;

        x.checked_add(width)?;
        y.checked_add(height)?;

        let width = LengthU32::new(width)?;
        let height = LengthU32::new(height)?;

        Some(ScreenIntRect { x, y, width, height })
    }

    /// Creates a new `ScreenIntRect`.
    #[inline]
    pub const fn from_xywh_safe(x: u32, y: u32, width: LengthU32, height: LengthU32) -> Self {
        ScreenIntRect { x, y, width, height }
    }

    /// Creates a new `ScreenIntRect` from x, y, width and height without checking them.
    ///
    /// # Safety
    ///
    /// `width` and `height` must be > 0.
    #[inline]
    pub const unsafe fn from_xywh_unchecked(x: u32, y: u32, width: u32, height: u32) -> Self {
        ScreenIntRect {
            x,
            y,
            width: LengthU32::new_unchecked(width),
            height: LengthU32::new_unchecked(height),
        }
    }

    /// Returns rect's X position.
    #[inline]
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Returns rect's Y position.
    #[inline]
    pub fn y(&self) -> u32 {
        self.y
    }

    /// Returns rect's width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns rect's height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Returns rect's width.
    #[inline]
    pub fn width_safe(&self) -> LengthU32 {
        self.width
    }

    /// Returns rect's height.
    #[inline]
    pub fn height_safe(&self) -> LengthU32 {
        self.height
    }

    /// Returns rect's left edge.
    #[inline]
    pub fn left(&self) -> u32 {
        self.x
    }

    /// Returns rect's top edge.
    #[inline]
    pub fn top(&self) -> u32 {
        self.y
    }

    /// Returns rect's right edge.
    ///
    /// The right edge is at least 1.
    #[inline]
    pub fn right(&self) -> u32 {
        // No overflow is guaranteed by constructors.
        self.x + self.width.get()
    }

    /// Returns rect's bottom edge.
    ///
    /// The bottom edge is at least 1.
    #[inline]
    pub fn bottom(&self) -> u32 {
        // No overflow is guaranteed by constructors.
        self.y + self.height.get()
    }

    /// Returns rect's right edge.
    ///
    /// The right edge is at least 1.
    #[inline]
    pub fn right_safe(&self) -> LengthU32 {
        // No overflow is guaranteed by constructors.
        unsafe {
            LengthU32::new_unchecked(self.x + self.width.get())
        }
    }

    /// Returns rect's bottom edge.
    ///
    /// The bottom edge is at least 1.
    #[inline]
    pub fn bottom_safe(&self) -> LengthU32 {
        // No overflow is guaranteed by constructors.
        unsafe {
            LengthU32::new_unchecked(self.y + self.height.get())
        }
    }

    /// Checks that the rect is completely includes `other` Rect.
    #[inline]
    pub fn contains(&self, other: &Self) -> bool {
        self.x <= other.x &&
        self.y <= other.y &&
        self.right() >= other.right() &&
        self.bottom() >= other.bottom()
    }

    /// Returns an intersection of two rectangles.
    ///
    /// Returns `None` otherwise.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);

        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        let w = right.checked_sub(left)?;
        let h = bottom.checked_sub(top)?;

        ScreenIntRect::from_xywh(left, top, w, h)
    }

    /// Returns rect's size.
    #[inline]
    pub fn size(&self) -> IntSize {
        IntSize::from_wh_safe(self.width, self.height)
    }

    /// Converts into a `IntRect`.
    #[inline]
    pub fn to_int_rect(&self) -> IntRect {
        // Everything is already checked by constructor.
        unsafe {
            IntRect::from_xywh_unchecked(
                self.x as i32,
                self.y as i32,
                self.width.get(),
                self.height.get(),
            )
        }
    }

    /// Converts into a `Rect`.
    #[inline]
    pub fn to_rect(&self) -> Rect {
        // Can't fail, because `ScreenIntRect` is always valid.
        // And u32 always fits into f32.
        unsafe {
            Rect::from_ltrb_unchecked(
                self.x as f32,
                self.y as f32,
                self.x as f32 + self.width.get() as f32,
                self.y as f32 + self.height.get() as f32,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        unsafe {
            assert_eq!(ScreenIntRect::from_xywh(0, 0, 0, 0), None);
            assert_eq!(ScreenIntRect::from_xywh(0, 0, 1, 0), None);
            assert_eq!(ScreenIntRect::from_xywh(0, 0, 0, 1), None);

            assert_eq!(ScreenIntRect::from_xywh(0, 0, std::u32::MAX, std::u32::MAX), None);
            assert_eq!(ScreenIntRect::from_xywh(0, 0, 1, std::u32::MAX), None);
            assert_eq!(ScreenIntRect::from_xywh(0, 0, std::u32::MAX, 1), None);

            assert_eq!(ScreenIntRect::from_xywh(std::u32::MAX, 0, 1, 1), None);
            assert_eq!(ScreenIntRect::from_xywh(0, std::u32::MAX, 1, 1), None);

            assert_eq!(ScreenIntRect::from_xywh(std::u32::MAX, std::u32::MAX, std::u32::MAX, std::u32::MAX), None);

            assert_eq!(ScreenIntRect::from_xywh(1, 2, 3, 4),
                       Some(ScreenIntRect::from_xywh_unchecked(1, 2, 3, 4)));

            let r = ScreenIntRect::from_xywh(1, 2, 3, 4).unwrap();
            assert_eq!(r.x(), 1);
            assert_eq!(r.y(), 2);
            assert_eq!(r.width(), 3);
            assert_eq!(r.height(), 4);
            assert_eq!(r.right(), 4);
            assert_eq!(r.bottom(), 6);
            assert_eq!(r.size(), IntSize::from_unchecked_wh(3, 4));
            // assert_eq!(r.to_rect(), Rect::from_xywh_unchecked(1.0, 2.0, 3.0, 4.0));

            {
                // No intersection.
                let r1 = ScreenIntRect::from_xywh(1, 2, 3, 4).unwrap();
                let r2 = ScreenIntRect::from_xywh(11, 12, 13, 14).unwrap();
                assert_eq!(r1.intersect(&r2), None);
            }

            {
                // Second inside the first one.
                let r1 = ScreenIntRect::from_xywh(1, 2, 30, 40).unwrap();
                let r2 = ScreenIntRect::from_xywh(11, 12, 13, 14).unwrap();
                assert_eq!(r1.intersect(&r2), ScreenIntRect::from_xywh(11, 12, 13, 14));
            }

            {
                // Partial overlap.
                let r1 = ScreenIntRect::from_xywh(1, 2, 30, 40).unwrap();
                let r2 = ScreenIntRect::from_xywh(11, 12, 50, 60).unwrap();
                assert_eq!(r1.intersect(&r2), ScreenIntRect::from_xywh(11, 12, 20, 30));
            }
        }
    }
}
