// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Unlike U16x16, F32x4 SIMD is 3x faster than a scalar version.

#[cfg(all(feature = "sse2", target_arch = "x86"))]
use std::arch::x86;
#[cfg(all(feature = "sse2", target_arch = "x86_64"))]
use std::arch::x86_64 as x86;
#[cfg(all(feature = "sse2", target_feature = "sse2"))]
use x86::{__m128, __m128i};

use super::{I32x4, U32x4};

#[cfg(all(feature = "sse2", target_feature = "sse2"))]
type Storage = __m128;

#[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
type Storage = [f32; 4];

#[derive(Copy, Clone)]
pub struct F32x4(pub Storage);

impl F32x4 {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            let vector = [a, b, c, d];
            F32x4(x86::_mm_loadu_ps(vector.as_ptr()))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([a, b, c, d])
        }
    }

    pub fn splat(x: f32) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_set1_ps(x))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([x, x, x, x])
        }
    }

    pub fn approx_recip(self) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_rcp_ps(self.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                1.0 / self.0[0],
                1.0 / self.0[1],
                1.0 / self.0[2],
                1.0 / self.0[3],
            ])
        }
    }

    pub fn approx_recip_sqrt(self) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_rsqrt_ps(self.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                1.0 / self.0[0].sqrt(),
                1.0 / self.0[1].sqrt(),
                1.0 / self.0[2].sqrt(),
                1.0 / self.0[3].sqrt(),
            ])
        }
    }

    pub fn sqrt(self) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_sqrt_ps(self.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.0[0].sqrt(),
                self.0[1].sqrt(),
                self.0[2].sqrt(),
                self.0[3].sqrt(),
            ])
        }
    }

    pub fn min(self, other: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_min_ps(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x().min(other.x()),
                self.y().min(other.y()),
                self.z().min(other.z()),
                self.w().min(other.w()),
            ])
        }
    }

    pub fn max(self, other: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_max_ps(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x().max(other.x()),
                self.y().max(other.y()),
                self.z().max(other.z()),
                self.w().max(other.w()),
            ])
        }
    }

    pub fn abs(&self) -> Self {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            let tmp = x86::_mm_srli_epi32(I32x4::splat(-1).0, 1);
            F32x4(x86::_mm_and_ps(x86::_mm_castsi128_ps(tmp), self.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x().abs(),
                self.y().abs(),
                self.z().abs(),
                self.w().abs(),
            ])
        }
    }

    pub fn normalize(&self) -> Self {
        self.max(F32x4::default()).min(F32x4::splat(1.0))
    }

    pub fn trunc(&self) -> I32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            I32x4(x86::_mm_cvttps_epi32(self.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            I32x4([
                self.x().trunc() as i32,
                self.y().trunc() as i32,
                self.z().trunc() as i32,
                self.w().trunc() as i32,
            ])
        }
    }

    pub fn floor(&self) -> Self {
        let roundtrip = self.trunc().to_f32x4();
        roundtrip - roundtrip.packed_gt(*self).if_then_else(F32x4::splat(1.0), F32x4::default())
    }

    pub fn to_u32x4_bitcast(&self) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(*(&self.0 as *const __m128 as *const __m128i))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        unsafe {
            U32x4(std::mem::transmute::<[f32; 4], [u32; 4]>(self.0))
        }
    }

    pub fn to_i32x4_round(self) -> I32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            I32x4(x86::_mm_cvtps_epi32(self.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            I32x4([
                self.x().round() as i32,
                self.y().round() as i32,
                self.z().round() as i32,
                self.w().round() as i32,
            ])
        }
    }

    pub fn packed_eq(self, other: F32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_castps_si128(x86::_mm_cmpeq_ps(self.0, other.0)))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                if self.0[0] == other.0[0] { !0 } else { 0 },
                if self.0[1] == other.0[1] { !0 } else { 0 },
                if self.0[2] == other.0[2] { !0 } else { 0 },
                if self.0[3] == other.0[3] { !0 } else { 0 },
            ])
        }
    }

    pub fn packed_ne(self, other: F32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_castps_si128(x86::_mm_cmpneq_ps(self.0, other.0)))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                if self.0[0] != other.0[0] { !0 } else { 0 },
                if self.0[1] != other.0[1] { !0 } else { 0 },
                if self.0[2] != other.0[2] { !0 } else { 0 },
                if self.0[3] != other.0[3] { !0 } else { 0 },
            ])
        }
    }

    pub fn packed_gt(self, other: F32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_castps_si128(x86::_mm_cmpgt_ps(self.0, other.0)))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                if self.0[0] > other.0[0] { !0 } else { 0 },
                if self.0[1] > other.0[1] { !0 } else { 0 },
                if self.0[2] > other.0[2] { !0 } else { 0 },
                if self.0[3] > other.0[3] { !0 } else { 0 },
            ])
        }
    }

    pub fn packed_ge(self, other: F32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_castps_si128(x86::_mm_cmpge_ps(self.0, other.0)))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                if self.0[0] >= other.0[0] { !0 } else { 0 },
                if self.0[1] >= other.0[1] { !0 } else { 0 },
                if self.0[2] >= other.0[2] { !0 } else { 0 },
                if self.0[3] >= other.0[3] { !0 } else { 0 },
            ])
        }
    }

    pub fn packed_le(self, other: F32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_castps_si128(x86::_mm_cmple_ps(self.0, other.0)))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                if self.0[0] <= other.0[0] { !0 } else { 0 },
                if self.0[1] <= other.0[1] { !0 } else { 0 },
                if self.0[2] <= other.0[2] { !0 } else { 0 },
                if self.0[3] <= other.0[3] { !0 } else { 0 },
            ])
        }
    }

    pub fn as_slice(&self) -> &[f32; 4] {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            &*(&self.0 as *const __m128 as *const [f32; 4])
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            &self.0
        }
    }

    pub fn x(&self) -> f32 { self.as_slice()[0] }
    pub fn y(&self) -> f32 { self.as_slice()[1] }
    pub fn z(&self) -> f32 { self.as_slice()[2] }
    pub fn w(&self) -> f32 { self.as_slice()[3] }
}

impl Default for F32x4 {
    fn default() -> Self {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_setzero_ps())
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([0.0, 0.0, 0.0, 0.0])
        }
    }
}

impl std::fmt::Debug for F32x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "F32x4({:?})", self.as_slice())
    }
}

impl PartialEq for F32x4 {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        {
            self.packed_eq(*other).all_true()
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            self.0[0] == other.0[0] &&
            self.0[1] == other.0[1] &&
            self.0[2] == other.0[2] &&
            self.0[3] == other.0[3]
        }
    }
}

impl std::ops::Add<F32x4> for F32x4 {
    type Output = F32x4;

    fn add(self, other: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_add_ps(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x() + other.x(),
                self.y() + other.y(),
                self.z() + other.z(),
                self.w() + other.w(),
            ])
        }
    }
}

impl std::ops::Sub<F32x4> for F32x4 {
    type Output = F32x4;

    fn sub(self, other: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_sub_ps(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x() - other.x(),
                self.y() - other.y(),
                self.z() - other.z(),
                self.w() - other.w(),
            ])
        }
    }
}

impl std::ops::Mul<F32x4> for F32x4 {
    type Output = F32x4;

    fn mul(self, other: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_mul_ps(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x() * other.x(),
                self.y() * other.y(),
                self.z() * other.z(),
                self.w() * other.w(),
            ])
        }
    }
}

impl std::ops::MulAssign for F32x4 {
    fn mul_assign(&mut self, other: F32x4) {
        *self = *self * other
    }
}

impl std::ops::Div<F32x4> for F32x4 {
    type Output = F32x4;

    fn div(self, other: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_div_ps(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x() / other.x(),
                self.y() / other.y(),
                self.z() / other.z(),
                self.w() / other.w(),
            ])
        }
    }
}
