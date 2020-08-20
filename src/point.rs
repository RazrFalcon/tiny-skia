// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use pathfinder_simd::default::F32x2;

/// A point.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    // TODO: should be finite?
    /// Creates a new `Point`.
    #[inline]
    pub fn from_xy(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    #[inline]
    pub(crate) fn from_f32x2(r: F32x2) -> Self {
        Point::from_xy(r[0], r[1])
    }

    #[inline]
    pub(crate) fn to_f32x2(&self) -> F32x2 {
        F32x2::new(self.x, self.y)
    }

    /// Creates a point at 0x0 position.
    #[inline]
    pub fn zero() -> Self {
        Point { x: 0.0, y: 0.0 }
    }

    /// Returns true if x and y are both zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    #[inline]
    fn add(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x + other.x,
            self.y + other.y,
        )
    }
}

impl std::ops::AddAssign for Point {
    #[inline]
    fn add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Mul for Point {
    type Output = Point;

    #[inline]
    fn mul(self, other: Point) -> Self::Output {
        Point::from_xy(
            self.x * other.x,
            self.y * other.y,
        )
    }
}

impl std::ops::MulAssign for Point {
    #[inline]
    fn mul_assign(&mut self, other: Point) {
        self.x *= other.x;
        self.y *= other.y;
    }
}
