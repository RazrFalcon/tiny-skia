// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::scalar::Scalar;

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
    fn saturate_from(mut x: f64) -> Self {
        x = if x < i32::MAX as f64 { x } else { i32::MAX as f64 };
        x = if x > i32::MIN as f64 { x } else { i32::MIN as f64 };
        x as i32
    }
}


pub trait SaturateRound<T>: SaturateCast<T> {
    fn saturate_floor(n: T) -> Self;
    fn saturate_ceil(n: T) -> Self;
    fn saturate_round(n: T) -> Self;
}

impl SaturateRound<f32> for i32 {
    #[inline]
    fn saturate_floor(x: f32) -> Self {
        Self::saturate_from(x.floor())
    }

    #[inline]
    fn saturate_ceil(x: f32) -> Self {
        Self::saturate_from(x.ceil())
    }

    #[inline]
    fn saturate_round(x: f32) -> Self {
        Self::saturate_from(x.floor() + 0.5)
    }
}

// f32 wrappers below were not part of Skia.


macro_rules! impl_debug_display {
    ($t:ident) => {
        impl std::fmt::Debug for $t {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.get())
            }
        }

        impl std::fmt::Display for $t {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.get())
            }
        }
    };
}


/// A float that is known to be finite.
///
/// Unlike `f32`, implements `Ord`, `PartialOrd` and `Hash`.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct FiniteF32(f32);

impl FiniteF32 {
    /// Creates a finite f32 number.
    ///
    /// Returns `None` for NaN and infinity.
    #[inline]
    pub fn new(n: f32) -> Option<Self> {
        if n.is_finite() {
            Some(FiniteF32(n))
        } else {
            None
        }
    }

    /// Creates a non-zero without checking the value.
    ///
    /// # Safety
    ///
    /// `n` must be finite.
    #[inline]
    pub const unsafe fn new_unchecked(n: f32) -> Self {
        FiniteF32(n)
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(&self) -> f32 {
        self.0
    }
}

impl Eq for FiniteF32 {}

impl PartialEq for FiniteF32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Ord for FiniteF32 {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0 < other.0 {
            std::cmp::Ordering::Less
        } else if self.0 > other.0 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for FiniteF32 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl_debug_display!(FiniteF32);


/// An immutable `f32` in a 0..=1 range.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
#[repr(transparent)]
pub struct NormalizedF32(FiniteF32);

impl NormalizedF32 {
    /// A NormalizedF32 value initialized with zero.
    pub const ZERO: Self = NormalizedF32(FiniteF32(0.0));
    /// A NormalizedF32 value initialized with one.
    pub const ONE: Self  = NormalizedF32(FiniteF32(1.0));

    /// Creates a `NormalizedF32` if the given value is in a 0..1 range.
    #[inline]
    pub fn new(n: f32) -> Option<Self> {
        if n.is_finite() && n >= 0.0 && n <= 1.0 {
            Some(NormalizedF32(FiniteF32(n)))
        } else {
            None
        }
    }

    /// Creates a new `NormalizedF32` without checking the value.
    ///
    /// # Safety
    ///
    /// `n` must be in 0..=1 range.
    #[inline]
    pub const unsafe fn new_unchecked(n: f32) -> Self {
        NormalizedF32(FiniteF32(n))
    }

    /// Creates a `NormalizedValue` clamping the given value to a 0..1 range.
    ///
    /// Returns zero in case of NaN or infinity.
    #[inline]
    pub fn new_bounded(n: f32) -> Self {
        if n.is_finite() {
            NormalizedF32(FiniteF32(n.bound(0.0, 1.0)))
        } else {
            Self::ZERO
        }
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(self) -> f32 {
        self.0.get()
    }

    /// Returns the value as a `FiniteF32`.
    #[inline]
    pub const fn get_finite(&self) -> FiniteF32 {
        self.0
    }
}

impl_debug_display!(NormalizedF32);


/// A float that is known not to equal zero.
///
/// Doesn't support NonNull memory layout optimization like `std` types.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct NonZeroF32(FiniteF32);

impl NonZeroF32 {
    /// Creates a non-zero if the given value is not zero.
    ///
    /// Returns `None` for NaN and infinity.
    #[inline]
    pub fn new(n: f32) -> Option<Self> {
        if n.is_finite() && n != 0.0 {
            Some(NonZeroF32(FiniteF32(n)))
        } else {
            None
        }
    }

    /// Creates a non-zero without checking the value.
    ///
    /// # Safety
    ///
    /// `n` must be finite and non-zero.
    #[inline]
    pub const unsafe fn new_unchecked(n: f32) -> Self {
        NonZeroF32(FiniteF32(n))
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(&self) -> f32 {
        self.0.get()
    }
}

impl_debug_display!(NonZeroF32);


/// A float that is known to be > 0.
///
/// Doesn't support NonNull memory layout optimization like `std` types.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct NonZeroPositiveF32(FiniteF32);

impl NonZeroPositiveF32 {
    /// Creates a new `NonZeroPositiveF32` if the given value is positive.
    ///
    /// Returns `None` for NaN and infinity.
    #[inline]
    pub fn new(n: f32) -> Option<Self> {
        if n.is_finite() && n > 0.0 {
            Some(NonZeroPositiveF32(FiniteF32(n)))
        } else {
            None
        }
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(&self) -> f32 {
        self.0.get()
    }
}

impl_debug_display!(NonZeroPositiveF32);
