// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::scalar::Scalar;

#[cfg(all(not(feature = "std"), feature = "libm"))]
use crate::scalar::FloatExt;

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
    fn saturate_floor(x: f32) -> Self {
        Self::saturate_from(x.floor())
    }

    fn saturate_ceil(x: f32) -> Self {
        Self::saturate_from(x.ceil())
    }

    fn saturate_round(x: f32) -> Self {
        Self::saturate_from(x.floor() + 0.5)
    }
}

/// Return the float as a 2s compliment int. Just to be used to compare floats
/// to each other or against positive float-bit-constants (like 0). This does
/// not return the int equivalent of the float, just something cheaper for
/// compares-only.
pub fn f32_as_2s_compliment(x: f32) -> i32 {
    sign_bit_to_2s_compliment(bytemuck::cast(x))
}

/// Convert a sign-bit int (i.e. float interpreted as int) into a 2s compliement
/// int. This also converts -0 (0x80000000) to 0. Doing this to a float allows
/// it to be compared using normal C operators (<, <=, etc.)
fn sign_bit_to_2s_compliment(mut x: i32) -> i32 {
    if x < 0 {
        x &= 0x7FFFFFFF;
        x = -x;
    }

    x
}


// f32 wrappers below were not part of Skia.


macro_rules! impl_debug_display {
    ($t:ident) => {
        impl core::fmt::Debug for $t {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "{}", self.get())
            }
        }

        impl core::fmt::Display for $t {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
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
    pub const FINITE_ZERO: FiniteF32 = FiniteF32(0.0);
    pub const FINITE_ONE: FiniteF32 = FiniteF32(1.0);

    /// Creates a finite f32 number.
    ///
    /// Returns `None` for NaN and infinity.
    pub fn new(n: f32) -> Option<Self> {
        if n.is_finite() {
            Some(FiniteF32(n))
        } else {
            None
        }
    }

    /// Returns the value as a primitive type.
    pub const fn get(&self) -> f32 {
        self.0
    }
}

impl Eq for FiniteF32 {}

impl PartialEq for FiniteF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Ord for FiniteF32 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.0 < other.0 {
            core::cmp::Ordering::Less
        } else if self.0 > other.0 {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for FiniteF32 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
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
    pub fn new(n: f32) -> Option<Self> {
        if n.is_finite() && n >= 0.0 && n <= 1.0 {
            Some(NormalizedF32(FiniteF32(n)))
        } else {
            None
        }
    }

    pub fn from_u8(n: u8) -> Self {
        NormalizedF32(FiniteF32(f32::from(n) / 255.0))
    }

    /// Creates a `NormalizedValue` clamping the given value to a 0..1 range.
    ///
    /// Returns zero in case of NaN or infinity.
    pub fn new_bounded(n: f32) -> Self {
        NormalizedF32(FiniteF32(n.bound(0.0, 1.0)))
    }

    /// Returns the value as a primitive type.
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


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
#[repr(transparent)]
pub struct NormalizedF32Exclusive(FiniteF32);

impl NormalizedF32Exclusive {
    // Just a random, valid numbers to init the array.
    // Will be overwritten anyway.
    // Perfectly safe.
    pub const ANY: Self = NormalizedF32Exclusive(FiniteF32(0.5));

    pub const HALF: Self = NormalizedF32Exclusive(FiniteF32(0.5));

    pub fn new(n: f32) -> Option<Self> {
        if n > 0.0 && n < 1.0 {
            // `n` is guarantee to be finite after the bounds check.
            Some(NormalizedF32Exclusive(FiniteF32(n)))
        } else {
            None
        }
    }

    pub fn new_bounded(n: f32) -> Self {
        let n = n.bound(core::f32::EPSILON, 1.0 - core::f32::EPSILON);
        // `n` is guarantee to be finite after clamping.
        debug_assert!(n.is_finite());
        NormalizedF32Exclusive(FiniteF32(n))
    }

    pub fn get(self) -> f32 {
        self.0.get()
    }

    pub fn to_normalized(self) -> NormalizedF32 {
        // NormalizedF32 is (0,1), while NormalizedF32 is [0,1], so it will always fit.
        NormalizedF32(self.0)
    }
}


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
    pub fn new(n: f32) -> Option<Self> {
        if n.is_finite() && n > 0.0 {
            Some(NonZeroPositiveF32(FiniteF32(n)))
        } else {
            None
        }
    }

    /// Returns the value as a primitive type.
    pub const fn get(&self) -> f32 {
        self.0.get()
    }
}

impl_debug_display!(NonZeroPositiveF32);
