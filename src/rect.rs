// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::cmp;

use crate::{IntRect, Point};

use crate::floating_point::{SaturateRound, FiniteF32};
use crate::wide::f32x4;

/// A rectangle defined by left, top, right and bottom edges.
///
/// Can have zero width and/or height. But not a negative one.
///
/// # Guarantees
///
/// - All values are finite.
/// - Left edge is <= right.
/// - Top edge is <= bottom.
/// - Width and height are <= f32::MAX.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rect {
    left: FiniteF32,
    top: FiniteF32,
    right: FiniteF32,
    bottom: FiniteF32,
}

impl Rect {
    /// Creates new `Rect`.
    #[inline]
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Option<Self> {
        let left = FiniteF32::new(left)?;
        let top = FiniteF32::new(top)?;
        let right = FiniteF32::new(right)?;
        let bottom = FiniteF32::new(bottom)?;

        if left.get() <= right.get() && top.get() <= bottom.get() {
            // Width and height must not overflow.
            checked_f32_sub(right.get(), left.get())?;
            checked_f32_sub(bottom.get(), top.get())?;

            Some(Rect { left, top, right, bottom })
        } else {
            None
        }
    }

    /// Creates new `Rect`.
    #[inline]
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Option<Self> {
        Rect::from_ltrb(x, y, w + x, h + y)
    }

    /// Creates new `Rect` without checking edges.
    ///
    /// # Safety
    ///
    /// All values must be finite.
    #[inline]
    pub const unsafe fn from_ltrb_unchecked(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Rect {
            left: FiniteF32::new_unchecked(left),
            top: FiniteF32::new_unchecked(top),
            right: FiniteF32::new_unchecked(right),
            bottom: FiniteF32::new_unchecked(bottom),
        }
    }

    /// Returns the left edge.
    #[inline]
    pub fn left(&self) -> f32 {
        self.left.get()
    }

    /// Returns the top edge.
    #[inline]
    pub fn top(&self) -> f32 {
        self.top.get()
    }

    /// Returns the right edge.
    #[inline]
    pub fn right(&self) -> f32 {
        self.right.get()
    }

    /// Returns the bottom edge.
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.bottom.get()
    }

    /// Returns rect's X position.
    #[inline]
    pub fn x(&self) -> f32 {
        self.left.get()
    }

    /// Returns rect's Y position.
    #[inline]
    pub fn y(&self) -> f32 {
        self.top.get()
    }

    /// Returns rect's width.
    #[inline]
    pub fn width(&self) -> f32 {
        self.right.get() - self.left.get()
    }

    /// Returns rect's height.
    #[inline]
    pub fn height(&self) -> f32 {
        self.bottom.get() - self.top.get()
    }

    /// Converts into an `IntRect` by adding 0.5 and discarding the fractional portion.
    ///
    /// Width and height are guarantee to be >= 1.
    #[inline]
    pub(crate) fn round(&self) -> IntRect {
        IntRect::from_xywh(
            i32::saturate_round(self.x()),
            i32::saturate_round(self.y()),
            cmp::max(1, i32::saturate_round(self.width()) as u32),
            cmp::max(1, i32::saturate_round(self.height()) as u32),
        ).unwrap()
    }

    /// Converts into an `IntRect` rounding outwards.
    ///
    /// Width and height are guarantee to be >= 1.
    #[inline]
    pub(crate) fn round_out(&self) -> IntRect {
        IntRect::from_xywh(
            i32::saturate_floor(self.x()),
            i32::saturate_floor(self.y()),
            cmp::max(1, i32::saturate_ceil(self.width()) as u32),
            cmp::max(1, i32::saturate_ceil(self.height()) as u32),
        ).unwrap()
    }

    /// Returns an intersection of two rectangles.
    ///
    /// Returns `None` otherwise.
    pub(crate) fn intersect(&self, other: &Self) -> Option<Self> {
        let left = self.x().max(other.x());
        let top = self.y().max(other.y());

        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Rect::from_ltrb(left, top, right, bottom)
    }

    /// Creates a Rect from Point array.
    ///
    /// Returns None if count is zero or if Point array contains an infinity or NaN.
    pub(crate) fn from_points(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut offset = 0;
        let mut min;
        let mut max;
        if points.len() & 1 != 0 {
            let pt = points[0];
            min = f32x4::from([pt.x, pt.y, pt.x, pt.y]);
            max = min;
            offset += 1;
        } else {
            let pt0 = points[0];
            let pt1 = points[1];
            min = f32x4::from([pt0.x, pt0.y, pt1.x, pt1.y]);
            max = min;
            offset += 2;
        }

        let mut accum = f32x4::default();
        while offset != points.len() {
            let pt0 = points[offset + 0];
            let pt1 = points[offset + 1];
            let xy = f32x4::from([pt0.x, pt0.y, pt1.x, pt1.y]);

            accum *= xy;
            min = min.min(xy);
            max = max.max(xy);
            offset += 2;
        }

        let all_finite = accum * f32x4::default() == f32x4::default();
        let min: [f32; 4] = min.into();
        let max: [f32; 4] = max.into();
        if all_finite {
            Rect::from_ltrb(
                min[0].min(min[2]),
                min[1].min(min[3]),
                max[0].max(max[2]),
                max[1].max(max[3]),
            )
        } else {
            None
        }
    }

    pub(crate) fn inset(&mut self, dx: f32, dy: f32) -> Option<Self> {
        Rect::from_ltrb(
            self.left() + dx,
            self.top() + dy,
            self.right() - dx,
            self.bottom() - dy,
        )
    }

    pub(crate) fn outset(&mut self, dx: f32, dy: f32) -> Option<Self> {
        self.inset(-dx, -dy)
    }
}

#[inline]
fn checked_f32_sub(a: f32, b: f32) -> Option<f32> {
    debug_assert!(a.is_finite());
    debug_assert!(b.is_finite());

    let n = a as f64 - b as f64;
    // Not sure if this is perfectly correct.
    if n > std::f32::MIN as f64 && n < std::f32::MAX as f64 {
        Some(n as f32)
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 5.0, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 10.0, 5.0), None);
        assert_eq!(Rect::from_ltrb(std::f32::NAN, 10.0, 10.0, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, std::f32::NAN, 10.0, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, std::f32::NAN, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 10.0, std::f32::NAN), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 10.0, std::f32::INFINITY), None);

        unsafe {
            assert_eq!(Rect::from_ltrb(10.0, 10.0, 10.0, 10.0),
                       Some(Rect::from_ltrb_unchecked(10.0, 10.0, 10.0, 10.0)));
        }

        let rect = Rect::from_ltrb(10.0, 20.0, 30.0, 40.0).unwrap();
        assert_eq!(rect.left(), 10.0);
        assert_eq!(rect.top(), 20.0);
        assert_eq!(rect.right(), 30.0);
        assert_eq!(rect.bottom(), 40.0);
        assert_eq!(rect.width(), 20.0);
        assert_eq!(rect.height(), 20.0);

        let rect = Rect::from_ltrb(-30.0, 20.0, -10.0, 40.0).unwrap();
        assert_eq!(rect.width(), 20.0);
        assert_eq!(rect.height(), 20.0);
    }
}
