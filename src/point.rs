// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::scalar::Scalar;
use crate::wide::F32x2;


/// Converts `&[Point; N]` into `&[f32; N*2]`.
macro_rules! points_to_f32s {
    ($pts:expr, $n:expr) => {
        unsafe { &*($pts as *const [Point; $n] as *const [f32; $n * 2]) }
    };
}


/// A point.
#[allow(missing_docs)]
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
    // TODO: should be finite?
    /// Creates a new `Point`.
    #[inline]
    pub fn from_xy(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    pub(crate) fn from_f32x2(r: F32x2) -> Self {
        Point::from_xy(r.x(), r.y())
    }

    pub(crate) fn to_f32x2(&self) -> F32x2 {
        F32x2::new(self.x, self.y)
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
        std::mem::swap(&mut self.x, &mut self.y);
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
fn set_point_length(pt: &mut Point, mut x: f32, mut y: f32, length: f32, orig_length: &mut Option<f32>) -> bool {
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

impl std::ops::Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x + other.x,
            self.y + other.y,
        )
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x - other.x,
            self.y - other.y,
        )
    }
}

impl std::ops::SubAssign for Point {
    fn sub_assign(&mut self, other: Point) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Mul for Point {
    type Output = Point;

    fn mul(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x * other.x,
            self.y * other.y,
        )
    }
}

impl std::ops::MulAssign for Point {
    fn mul_assign(&mut self, other: Point) {
        self.x *= other.x;
        self.y *= other.y;
    }
}
