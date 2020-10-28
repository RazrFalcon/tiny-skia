// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::convert::TryFrom;

use crate::{LengthU32, Rect};

use crate::screen_int_rect::ScreenIntRect;

/// An integer rectangle.
///
/// # Guarantees
///
/// - Width and height are in 1..=i32::MAX range.
/// - x+width and y+height does not overflow.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct IntRect {
    x: i32,
    y: i32,
    width: LengthU32,
    height: LengthU32,
}

impl IntRect {
    /// Creates a new `IntRect`.
    #[inline]
    pub fn from_xywh(x: i32, y: i32, width: u32, height: u32) -> Option<Self> {
        x.checked_add(i32::try_from(width).ok()?)?;
        y.checked_add(i32::try_from(height).ok()?)?;

        Some(IntRect {
            x,
            y,
            width: LengthU32::new(width)?,
            height: LengthU32::new(height)?,
        })
    }

    /// Creates a new `IntRect` without checking values.
    ///
    /// # Safety
    ///
    /// `width` and `height` must be > 0.
    #[inline]
    pub const unsafe fn from_xywh_unchecked(x: i32, y: i32, width: u32, height: u32) -> Self {
        IntRect {
            x,
            y,
            width: LengthU32::new_unchecked(width),
            height: LengthU32::new_unchecked(height),
        }
    }

    /// Creates a new `IntRect`.
    pub fn from_ltrb(left: i32, top: i32, right: i32, bottom: i32) -> Option<Self> {
        let width = u32::try_from(right.checked_sub(left)?).ok()?;
        let height = u32::try_from(bottom.checked_sub(top)?).ok()?;
        IntRect::from_xywh(left, top, width, height)
    }

    /// Returns rect's X position.
    #[inline]
    pub fn x(&self) -> i32 {
        self.x
    }

    /// Returns rect's Y position.
    #[inline]
    pub fn y(&self) -> i32 {
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

    /// Returns rect's left edge.
    #[inline]
    pub fn left(&self) -> i32 {
        self.x
    }

    /// Returns rect's top edge.
    #[inline]
    pub fn top(&self) -> i32 {
        self.y
    }

    /// Returns rect's right edge.
    #[inline]
    pub fn right(&self) -> i32 {
        // No overflow is guaranteed by constructors.
        self.x + self.width.get() as i32
    }

    /// Returns rect's bottom edge.
    #[inline]
    pub fn bottom(&self) -> i32 {
        // No overflow is guaranteed by constructors.
        self.y + self.height.get() as i32
    }

    /// Checks that the rect is completely includes `other` Rect.
    #[inline]
    pub(crate) fn contains(&self, other: &Self) -> bool {
        self.x <= other.x &&
        self.y <= other.y &&
        self.right() >= other.right() &&
        self.bottom() >= other.bottom()
    }

    /// Returns an intersection of two rectangles.
    ///
    /// Returns `None` otherwise.
    pub(crate) fn intersect(&self, other: &Self) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);

        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        let w = u32::try_from(right.checked_sub(left)?).ok()?;
        let h = u32::try_from(bottom.checked_sub(top)?).ok()?;

        IntRect::from_xywh(left, top, w, h)
    }

    /// Insets the rectangle.
    pub(crate) fn inset(&self, dx: i32, dy: i32) -> Option<Self> {
        IntRect::from_ltrb(
            self.left() + dx,
            self.top() + dy,
            self.right() - dx,
            self.bottom() - dy,
        )
    }

    /// Outsets the rectangle.
    pub(crate) fn make_outset(&self, dx: i32, dy: i32) -> Option<Self> {
        IntRect::from_ltrb(
            self.left().saturating_sub(dx),
            self.top().saturating_sub(dy),
            self.right().saturating_add(dx),
            self.bottom().saturating_add(dy),
        )
    }

    /// Converts into `Rect`.
    #[inline]
    pub fn to_rect(&self) -> Rect {
        // Can't fail, because `IntRect` is always valid.
        unsafe {
            Rect::from_ltrb_unchecked(
                self.x as f32,
                self.y as f32,
                self.x as f32 + self.width.get() as f32,
                self.y as f32 + self.height.get() as f32,
            )
        }
    }

    /// Converts into `ScreenIntRect`.
    ///
    /// # Checks
    ///
    /// - x >= 0
    /// - y >= 0
    #[inline]
    pub(crate) fn to_screen_int_rect(&self) -> Option<ScreenIntRect> {
        let x = u32::try_from(self.x).ok()?;
        let y = u32::try_from(self.y).ok()?;
        Some(ScreenIntRect::from_xywh_safe(x, y, self.width, self.height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(IntRect::from_xywh(0, 0, 0, 0), None);
        assert_eq!(IntRect::from_xywh(0, 0, 1, 0), None);
        assert_eq!(IntRect::from_xywh(0, 0, 0, 1), None);

        assert_eq!(IntRect::from_xywh(0, 0, std::u32::MAX, std::u32::MAX), None);
        assert_eq!(IntRect::from_xywh(0, 0, 1, std::u32::MAX), None);
        assert_eq!(IntRect::from_xywh(0, 0, std::u32::MAX, 1), None);

        assert_eq!(IntRect::from_xywh(std::i32::MAX, 0, 1, 1), None);
        assert_eq!(IntRect::from_xywh(0, std::i32::MAX, 1, 1), None);

        let r = IntRect::from_xywh(1, 2, 3, 4).unwrap();
        assert_eq!(r.to_screen_int_rect().unwrap(), ScreenIntRect::from_xywh(1, 2, 3, 4).unwrap());

        let r = IntRect::from_xywh(-1, -1, 3, 4).unwrap();
        assert_eq!(r.to_screen_int_rect(), None);

        {
            // No intersection.
            let r1 = IntRect::from_xywh(1, 2, 3, 4).unwrap();
            let r2 = IntRect::from_xywh(11, 12, 13, 14).unwrap();
            assert_eq!(r1.intersect(&r2), None);
        }

        {
            // Second inside the first one.
            let r1 = IntRect::from_xywh(1, 2, 30, 40).unwrap();
            let r2 = IntRect::from_xywh(11, 12, 13, 14).unwrap();
            assert_eq!(r1.intersect(&r2), IntRect::from_xywh(11, 12, 13, 14));
        }

        {
            // Partial overlap.
            let r1 = IntRect::from_xywh(1, 2, 30, 40).unwrap();
            let r2 = IntRect::from_xywh(11, 12, 50, 60).unwrap();
            assert_eq!(r1.intersect(&r2), IntRect::from_xywh(11, 12, 20, 30));
        }
    }
}
