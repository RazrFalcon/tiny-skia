// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use core::convert::TryFrom;

use crate::LengthU32;
use crate::floating_point::{SaturateRound, FiniteF32};
use crate::scalar::Scalar;
use crate::wide::{f32x2, f32x4};

#[cfg(all(not(feature = "std"), feature = "libm"))]
use crate::scalar::FloatExt;

/// A point.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for Point {
    #[inline]
    fn from(v: (f32, f32)) -> Self {
        Point { x: v.0, y: v.1 }
    }
}

impl Point {
    /// Creates a new `Point`.
    pub fn from_xy(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    pub(crate) fn from_f32x2(r: f32x2) -> Self {
        Point::from_xy(r.x(), r.y())
    }

    pub(crate) fn to_f32x2(&self) -> f32x2 {
        f32x2::new(self.x, self.y)
    }

    /// Creates a point at 0x0 position.
    pub fn zero() -> Self {
        Point { x: 0.0, y: 0.0 }
    }

    /// Returns true if x and y are both zero.
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    /// Returns true if both x and y are measurable values.
    ///
    /// Both values are other than infinities and NaN.
    pub(crate) fn is_finite(&self) -> bool {
        (self.x * self.y).is_finite()
    }

    pub(crate) fn almost_equal(&self, other: Point) -> bool {
        !(*self - other).can_normalize()
    }

    pub(crate) fn equals_within_tolerance(&self, other: Point, tolerance: f32) -> bool {
        (self.x - other.x).is_nearly_zero_within_tolerance(tolerance) &&
            (self.y - other.y).is_nearly_zero_within_tolerance(tolerance)
    }

    /// Scales (fX, fY) so that length() returns one, while preserving ratio of fX to fY,
    /// if possible.
    ///
    /// If prior length is nearly zero, sets vector to (0, 0) and returns
    /// false; otherwise returns true.
    pub(crate) fn normalize(&mut self) -> bool {
        self.set_length_from(self.x, self.y, 1.0)
    }

    /// Sets vector to (x, y) scaled so length() returns one, and so that (x, y)
    /// is proportional to (x, y).
    ///
    /// If (x, y) length is nearly zero, sets vector to (0, 0) and returns false;
    /// otherwise returns true.
    pub(crate) fn set_normalize(&mut self, x: f32, y: f32) -> bool {
        self.set_length_from(x, y, 1.0)
    }

    pub(crate) fn can_normalize(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && (self.x != 0.0 || self.y != 0.0)
    }

    /// Returns the Euclidean distance from origin.
    pub(crate) fn length(&self) -> f32 {
        let mag2 = self.x * self.x + self.y * self.y;
        if mag2.is_finite() {
            mag2.sqrt()
        } else {
            let xx = f64::from(self.x);
            let yy = f64::from(self.y);
            (xx * xx + yy * yy).sqrt() as f32
        }
    }

    /// Scales vector so that distanceToOrigin() returns length, if possible.
    ///
    /// If former length is nearly zero, sets vector to (0, 0) and return false;
    /// otherwise returns true.
    pub(crate) fn set_length(&mut self, length: f32) -> bool {
        self.set_length_from(self.x, self.y, length)
    }

    /// Sets vector to (x, y) scaled to length, if possible.
    ///
    /// If former length is nearly zero, sets vector to (0, 0) and return false;
    /// otherwise returns true.
    pub(crate) fn set_length_from(&mut self, x: f32, y: f32, length: f32) -> bool {
        set_point_length(self, x, y, length, &mut None)
    }

    /// Returns the Euclidean distance from origin.
    pub(crate) fn distance(&self, other: Point) -> f32 {
        (*self - other).length()
    }

    /// Returns the dot product of two points.
    pub(crate) fn dot(&self, other: Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the cross product of vector and vec.
    ///
    /// Vector and vec form three-dimensional vectors with z-axis value equal to zero.
    /// The cross product is a three-dimensional vector with x-axis and y-axis values
    /// equal to zero. The cross product z-axis component is returned.
    pub(crate) fn cross(&self, other: Point) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub(crate) fn distance_to_sqd(&self, pt: Point) -> f32 {
        let dx = self.x - pt.x;
        let dy = self.y - pt.y;
        dx * dx + dy * dy
    }

    pub(crate) fn length_sqd(&self) -> f32 {
        self.dot(*self)
    }

    /// Scales Point in-place by scale.
    pub(crate) fn scale(&mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
    }

    pub(crate) fn scaled(&self, scale: f32) -> Self {
        Point::from_xy(self.x * scale, self.y * scale)
    }

    pub(crate) fn swap_coords(&mut self) {
        core::mem::swap(&mut self.x, &mut self.y);
    }

    pub(crate) fn rotate_cw(&mut self) {
        self.swap_coords();
        self.x = -self.x;
    }

    pub(crate) fn rotate_ccw(&mut self) {
        self.swap_coords();
        self.y = -self.y;
    }
}

// We have to worry about 2 tricky conditions:
// 1. underflow of mag2 (compared against nearlyzero^2)
// 2. overflow of mag2 (compared w/ isfinite)
//
// If we underflow, we return false. If we overflow, we compute again using
// doubles, which is much slower (3x in a desktop test) but will not overflow.
fn set_point_length(
    pt: &mut Point,
    mut x: f32,
    mut y: f32,
    length: f32,
    orig_length: &mut Option<f32>,
) -> bool {
    // our mag2 step overflowed to infinity, so use doubles instead.
    // much slower, but needed when x or y are very large, other wise we
    // divide by inf. and return (0,0) vector.
    let xx = x as f64;
    let yy = y as f64;
    let dmag = (xx * xx + yy * yy).sqrt();
    let dscale = length as f64 / dmag;
    x *= dscale as f32;
    y *= dscale as f32;

    // check if we're not finite, or we're zero-length
    if !x.is_finite() || !y.is_finite() || (x == 0.0 && y == 0.0) {
        *pt = Point::zero();
        return false;
    }

    let mut mag = 0.0;
    if orig_length.is_some() {
        mag = dmag as f32;
    }

    *pt = Point::from_xy(x, y);

    if orig_length.is_some() {
        *orig_length = Some(mag);
    }

    true
}

impl core::ops::Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl core::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x + other.x,
            self.y + other.y,
        )
    }
}

impl core::ops::AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl core::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x - other.x,
            self.y - other.y,
        )
    }
}

impl core::ops::SubAssign for Point {
    fn sub_assign(&mut self, other: Point) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl core::ops::Mul for Point {
    type Output = Point;

    fn mul(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x * other.x,
            self.y * other.y,
        )
    }
}

impl core::ops::MulAssign for Point {
    fn mul_assign(&mut self, other: Point) {
        self.x *= other.x;
        self.y *= other.y;
    }
}


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
    pub fn from_wh(width: u32, height: u32) -> Option<Self> {
        Some(IntSize {
            width: LengthU32::new(width)?,
            height: LengthU32::new(height)?,
        })
    }

    /// Returns width.
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns height.
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Converts the current size into a `IntRect` at a provided position.
    pub fn to_int_rect(&self, x: i32, y: i32) -> IntRect {
        IntRect::from_xywh(x, y, self.width.get(), self.height.get()).unwrap()
    }

    /// Converts the current size into a `IntRect` at a provided position.
    pub(crate) fn to_screen_int_rect(&self, x: u32, y: u32) -> ScreenIntRect {
        ScreenIntRect::from_xywh_safe(x, y, self.width, self.height)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_size_tests() {
        assert_eq!(IntSize::from_wh(0, 0), None);
        assert_eq!(IntSize::from_wh(1, 0), None);
        assert_eq!(IntSize::from_wh(0, 1), None);

        let size = IntSize::from_wh(3, 4).unwrap();
        assert_eq!(size.to_int_rect(1, 2), IntRect::from_xywh(1, 2, 3, 4).unwrap());
        assert_eq!(size.to_screen_int_rect(1, 2), ScreenIntRect::from_xywh(1, 2, 3, 4).unwrap());
    }
}


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

    /// Creates a new `IntRect`.
    pub fn from_ltrb(left: i32, top: i32, right: i32, bottom: i32) -> Option<Self> {
        let width = u32::try_from(right.checked_sub(left)?).ok()?;
        let height = u32::try_from(bottom.checked_sub(top)?).ok()?;
        IntRect::from_xywh(left, top, width, height)
    }

    /// Returns rect's X position.
    pub fn x(&self) -> i32 {
        self.x
    }

    /// Returns rect's Y position.
    pub fn y(&self) -> i32 {
        self.y
    }

    /// Returns rect's width.
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns rect's height.
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Returns rect's left edge.
    pub fn left(&self) -> i32 {
        self.x
    }

    /// Returns rect's top edge.
    pub fn top(&self) -> i32 {
        self.y
    }

    /// Returns rect's right edge.
    pub fn right(&self) -> i32 {
        // No overflow is guaranteed by constructors.
        self.x + self.width.get() as i32
    }

    /// Returns rect's bottom edge.
    pub fn bottom(&self) -> i32 {
        // No overflow is guaranteed by constructors.
        self.y + self.height.get() as i32
    }

    /// Returns rect's size.
    pub(crate) fn size(&self) -> IntSize {
        IntSize {
            width: self.width,
            height: self.height,
        }
    }

    /// Checks that the rect is completely includes `other` Rect.
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
    pub fn to_rect(&self) -> Rect {
        // Can't fail, because `IntRect` is always valid.
        Rect::from_ltrb(
            self.x as f32,
            self.y as f32,
            self.x as f32 + self.width.get() as f32,
            self.y as f32 + self.height.get() as f32,
        ).unwrap()
    }

    /// Converts into `ScreenIntRect`.
    ///
    /// # Checks
    ///
    /// - x >= 0
    /// - y >= 0
    pub(crate) fn to_screen_int_rect(&self) -> Option<ScreenIntRect> {
        let x = u32::try_from(self.x).ok()?;
        let y = u32::try_from(self.y).ok()?;
        Some(ScreenIntRect::from_xywh_safe(x, y, self.width, self.height))
    }
}

#[cfg(test)]
mod int_rect_tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(IntRect::from_xywh(0, 0, 0, 0), None);
        assert_eq!(IntRect::from_xywh(0, 0, 1, 0), None);
        assert_eq!(IntRect::from_xywh(0, 0, 0, 1), None);

        assert_eq!(IntRect::from_xywh(0, 0, core::u32::MAX, core::u32::MAX), None);
        assert_eq!(IntRect::from_xywh(0, 0, 1, core::u32::MAX), None);
        assert_eq!(IntRect::from_xywh(0, 0, core::u32::MAX, 1), None);

        assert_eq!(IntRect::from_xywh(core::i32::MAX, 0, 1, 1), None);
        assert_eq!(IntRect::from_xywh(0, core::i32::MAX, 1, 1), None);

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
    pub const fn from_xywh_safe(x: u32, y: u32, width: LengthU32, height: LengthU32) -> Self {
        ScreenIntRect { x, y, width, height }
    }

    /// Returns rect's X position.
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Returns rect's Y position.
    pub fn y(&self) -> u32 {
        self.y
    }

    /// Returns rect's width.
    pub fn width(&self) -> u32 {
        self.width.get()
    }

    /// Returns rect's height.
    pub fn height(&self) -> u32 {
        self.height.get()
    }

    /// Returns rect's width.
    pub fn width_safe(&self) -> LengthU32 {
        self.width
    }

    /// Returns rect's left edge.
    pub fn left(&self) -> u32 {
        self.x
    }

    /// Returns rect's top edge.
    pub fn top(&self) -> u32 {
        self.y
    }

    /// Returns rect's right edge.
    ///
    /// The right edge is at least 1.
    pub fn right(&self) -> u32 {
        // No overflow is guaranteed by constructors.
        self.x + self.width.get()
    }

    /// Returns rect's bottom edge.
    ///
    /// The bottom edge is at least 1.
    pub fn bottom(&self) -> u32 {
        // No overflow is guaranteed by constructors.
        self.y + self.height.get()
    }

    /// Returns rect's size.
    pub fn size(&self) -> IntSize {
        IntSize {
            width: self.width,
            height: self.height,
        }
    }

    /// Checks that the rect is completely includes `other` Rect.
    pub fn contains(&self, other: &Self) -> bool {
        self.x <= other.x &&
            self.y <= other.y &&
            self.right() >= other.right() &&
            self.bottom() >= other.bottom()
    }

    /// Converts into a `IntRect`.
    pub fn to_int_rect(&self) -> IntRect {
        // Everything is already checked by constructors.
        IntRect::from_xywh(
            self.x as i32,
            self.y as i32,
            self.width.get(),
            self.height.get(),
        ).unwrap()
    }

    /// Converts into a `Rect`.
    pub fn to_rect(&self) -> Rect {
        // Can't fail, because `ScreenIntRect` is always valid.
        // And u32 always fits into f32.
        Rect::from_ltrb(
            self.x as f32,
            self.y as f32,
            self.x as f32 + self.width.get() as f32,
            self.y as f32 + self.height.get() as f32,
        ).unwrap()
    }
}

#[cfg(test)]
mod screen_int_rect_tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(ScreenIntRect::from_xywh(0, 0, 0, 0), None);
        assert_eq!(ScreenIntRect::from_xywh(0, 0, 1, 0), None);
        assert_eq!(ScreenIntRect::from_xywh(0, 0, 0, 1), None);

        assert_eq!(ScreenIntRect::from_xywh(0, 0, core::u32::MAX, core::u32::MAX), None);
        assert_eq!(ScreenIntRect::from_xywh(0, 0, 1, core::u32::MAX), None);
        assert_eq!(ScreenIntRect::from_xywh(0, 0, core::u32::MAX, 1), None);

        assert_eq!(ScreenIntRect::from_xywh(core::u32::MAX, 0, 1, 1), None);
        assert_eq!(ScreenIntRect::from_xywh(0, core::u32::MAX, 1, 1), None);

        assert_eq!(ScreenIntRect::from_xywh(core::u32::MAX, core::u32::MAX, core::u32::MAX, core::u32::MAX), None);

        let r = ScreenIntRect::from_xywh(1, 2, 3, 4).unwrap();
        assert_eq!(r.x(), 1);
        assert_eq!(r.y(), 2);
        assert_eq!(r.width(), 3);
        assert_eq!(r.height(), 4);
        assert_eq!(r.right(), 4);
        assert_eq!(r.bottom(), 6);
    }
}


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
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Option<Self> {
        Rect::from_ltrb(x, y, w + x, h + y)
    }

    /// Returns the left edge.
    pub fn left(&self) -> f32 {
        self.left.get()
    }

    /// Returns the top edge.
    pub fn top(&self) -> f32 {
        self.top.get()
    }

    /// Returns the right edge.
    pub fn right(&self) -> f32 {
        self.right.get()
    }

    /// Returns the bottom edge.
    pub fn bottom(&self) -> f32 {
        self.bottom.get()
    }

    /// Returns rect's X position.
    pub fn x(&self) -> f32 {
        self.left.get()
    }

    /// Returns rect's Y position.
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
    pub fn round(&self) -> IntRect {
        IntRect::from_xywh(
            i32::saturate_round(self.x()),
            i32::saturate_round(self.y()),
            core::cmp::max(1, i32::saturate_round(self.width()) as u32),
            core::cmp::max(1, i32::saturate_round(self.height()) as u32),
        ).unwrap()
    }

    /// Converts into an `IntRect` rounding outwards.
    ///
    /// Width and height are guarantee to be >= 1.
    pub(crate) fn round_out(&self) -> IntRect {
        IntRect::from_xywh(
            i32::saturate_floor(self.x()),
            i32::saturate_floor(self.y()),
            core::cmp::max(1, i32::saturate_ceil(self.width()) as u32),
            core::cmp::max(1, i32::saturate_ceil(self.height()) as u32),
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

fn checked_f32_sub(a: f32, b: f32) -> Option<f32> {
    debug_assert!(a.is_finite());
    debug_assert!(b.is_finite());

    let n = a as f64 - b as f64;
    // Not sure if this is perfectly correct.
    if n > core::f32::MIN as f64 && n < core::f32::MAX as f64 {
        Some(n as f32)
    } else {
        None
    }
}


#[cfg(test)]
mod rect_tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 5.0, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 10.0, 5.0), None);
        assert_eq!(Rect::from_ltrb(core::f32::NAN, 10.0, 10.0, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, core::f32::NAN, 10.0, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, core::f32::NAN, 10.0), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 10.0, core::f32::NAN), None);
        assert_eq!(Rect::from_ltrb(10.0, 10.0, 10.0, core::f32::INFINITY), None);

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
