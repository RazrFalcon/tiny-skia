// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub const FLOAT_PI: f32 = 3.14159265;

const MAX_I32_FITS_IN_F32: f32 = 2147483520.0;
const MIN_I32_FITS_IN_F32: f32 = -MAX_I32_FITS_IN_F32;

// TODO: is there an std alternative?

pub trait SaturateCast<T>: Sized {
    fn saturate_from(n: T) -> Self;
}

impl SaturateCast<f32> for i32 {
    /// Return the closest int for the given float.
    ///
    /// Returns MAX_I32_FITS_IN_F32 for NaN.
    #[inline]
    fn saturate_from(mut x: f32) -> Self {
        x = if x < MAX_I32_FITS_IN_F32 { x } else { MAX_I32_FITS_IN_F32 };
        x = if x > MIN_I32_FITS_IN_F32 { x } else { MIN_I32_FITS_IN_F32 };
        x as i32
    }
}

impl SaturateCast<f64> for i32 {
    /// Return the closest int for the given double.
    ///
    /// Returns i32::MAX for NaN.
    #[inline]
    fn saturate_from(mut x: f64) -> Self {
        x = if x < i32::MAX as f64 { x } else { i32::MAX as f64 };
        x = if x > i32::MIN as f64 { x } else { i32::MIN as f64 };
        x as i32
    }
}
