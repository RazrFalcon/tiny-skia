// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use float_cmp::ApproxEqUlps;

pub const SCALAR_MAX: f32           = 3.402823466e+38;
pub const SCALAR_NEARLY_ZERO: f32   = 1.0 / (1 << 12) as f32;
pub const SCALAR_ROOT_2_OVER_2: f32 = 0.707106781;

pub trait Scalar {
    fn half(self) -> Self;
    fn ave(self, other: Self) -> Self;
    fn sqr(self) -> Self;
    fn invert(self) -> Self;
    fn bound(self, min: Self, max: Self) -> Self;
    fn is_nearly_zero(self) -> bool;
    fn is_nearly_zero_within_tolerance(self, tolerance: Self) -> bool;
    fn almost_dequal_ulps(self, other: Self) -> bool;
}

impl Scalar for f32 {
    fn half(self) -> f32 {
        self * 0.5
    }

    fn ave(self, other: Self) -> f32 {
        (self + other) * 0.5
    }

    fn sqr(self) -> f32 {
        self * self
    }

    fn invert(self) -> f32 {
        1.0 / self
    }

    // Works just like SkTPin, returning `max` for NaN/inf
    fn bound(self, min: Self, max: Self) -> Self {
        max.min(self).max(min)
    }

    fn is_nearly_zero(self) -> bool {
        self.is_nearly_zero_within_tolerance(SCALAR_NEARLY_ZERO)
    }

    fn is_nearly_zero_within_tolerance(self, tolerance: Self) -> bool {
        debug_assert!(tolerance >= 0.0);
        self.abs() <= tolerance
    }

    // From SkPathOpsTypes.
    fn almost_dequal_ulps(self, other: Self) -> bool {
        self.approx_eq_ulps(&other, 16)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bound() {
        assert_eq!(std::f32::NAN.bound(0.0, 1.0), 1.0);
        assert_eq!(std::f32::INFINITY.bound(0.0, 1.0), 1.0);
        assert_eq!(std::f32::NEG_INFINITY.bound(0.0, 1.0), 0.0);
        assert_eq!(std::f32::EPSILON.bound(0.0, 1.0), std::f32::EPSILON);
        assert_eq!(0.5.bound(0.0, 1.0), 0.5);
        assert_eq!((-1.0).bound(0.0, 1.0), 0.0);
        assert_eq!(2.0.bound(0.0, 1.0), 1.0);
    }
}
