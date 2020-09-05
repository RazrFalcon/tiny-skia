// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[cfg(all(feature = "sse2", target_arch = "x86"))]
use std::arch::x86;
#[cfg(all(feature = "sse2", target_arch = "x86_64"))]
use std::arch::x86_64 as x86;
#[cfg(all(feature = "sse2", target_feature = "sse2"))]
use x86::{__m128, __m128i};

use super::F32x4;

#[cfg(all(feature = "sse2", target_feature = "sse2"))]
type Storage = __m128i;

#[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
type Storage = [u32; 4];

#[derive(Copy, Clone)]
pub struct U32x4(pub Storage);

impl U32x4 {
    pub fn new(a: u32, b: u32, c: u32, d: u32) -> Self {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            let vector = [a, b, c, d];
            U32x4(x86::_mm_loadu_si128(vector.as_ptr() as *const __m128i))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([a, b, c, d])
        }
    }

    pub fn splat(n: u32) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_set1_epi32(n as i32))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([n, n, n, n])
        }
    }

    pub fn as_slice(&self) -> &[u32; 4] {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            &*(&self.0 as *const __m128i as *const [u32; 4])
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            &self.0
        }
    }

    /// Returns true if all four booleans in this vector are true.
    ///
    /// The result is *undefined* if all four values in this vector are not booleans. A boolean is
    /// a value with all bits set or all bits clear (i.e. !0 or 0).
    #[cfg(all(feature = "sse2", target_feature = "sse2"))]
    pub fn all_true(self) -> bool {
        unsafe { x86::_mm_movemask_ps(x86::_mm_castsi128_ps(self.0)) == 0x0f }
    }

    pub fn if_then_else(&self, t: F32x4, e: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            // Do not use _mm_cvtepi32_ps here. We need `self` as bits, not as float.
            let c = *(&self.0 as *const __m128i as *const __m128);
            F32x4(x86::_mm_or_ps(x86::_mm_and_ps(c, t.0), x86::_mm_andnot_ps(c, e.0)))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                if self.0[0] != 0 { t.0[0] } else { e.0[0] },
                if self.0[1] != 0 { t.0[1] } else { e.0[1] },
                if self.0[2] != 0 { t.0[2] } else { e.0[2] },
                if self.0[3] != 0 { t.0[3] } else { e.0[3] },
            ])
        }
    }

    pub fn to_f32x4_bitcast(&self) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(*(&self.0 as *const __m128i as *const __m128))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        unsafe {
            F32x4(std::mem::transmute::<[u32; 4], [f32; 4]>(self.0))
        }
    }

    pub fn x(&self) -> u32 { self.as_slice()[0] }
    pub fn y(&self) -> u32 { self.as_slice()[1] }
    pub fn z(&self) -> u32 { self.as_slice()[2] }
    pub fn w(&self) -> u32 { self.as_slice()[3] }
}

impl Default for U32x4 {
    fn default() -> Self {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_setzero_si128())
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([0, 0, 0, 0])
        }
    }
}

impl std::ops::Not for U32x4 {
    type Output = U32x4;

    #[inline]
    fn not(self) -> U32x4 {
        self ^ U32x4::splat(!0)
    }
}

impl std::ops::Add<U32x4> for U32x4 {
    type Output = U32x4;

    fn add(self, other: U32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_add_epi32(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                self.x() + other.x(),
                self.y() + other.y(),
                self.z() + other.z(),
                self.w() + other.w(),
            ])
        }
    }
}

impl std::ops::Mul<U32x4> for U32x4 {
    type Output = U32x4;

    #[inline]
    fn mul(self, other: U32x4) -> U32x4 {
        // U32x4 multiplication is available only since SSE 4.1
        // TODO: use mullo32 from SkNx
        U32x4::new(
            self.x() * other.x(),
            self.y() * other.y(),
            self.z() * other.z(),
            self.w() * other.w(),
        )
    }
}

impl std::ops::BitAnd<U32x4> for U32x4 {
    type Output = U32x4;

    fn bitand(self, other: U32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_and_si128(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                self.x() & other.x(),
                self.y() & other.y(),
                self.z() & other.z(),
                self.w() & other.w(),
            ])
        }
    }
}

impl std::ops::BitOr<U32x4> for U32x4 {
    type Output = U32x4;

    fn bitor(self, other: U32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_or_si128(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                self.x() | other.x(),
                self.y() | other.y(),
                self.z() | other.z(),
                self.w() | other.w(),
            ])
        }
    }
}

impl std::ops::BitXor<U32x4> for U32x4 {
    type Output = U32x4;

    #[inline]
    fn bitxor(self, other: U32x4) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            U32x4(x86::_mm_xor_si128(self.0, other.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            U32x4([
                self.x() ^ other.x(),
                self.y() ^ other.y(),
                self.z() ^ other.z(),
                self.w() ^ other.w(),
            ])
        }
    }
}

impl std::fmt::Debug for U32x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "U32x4({:?})", self.as_slice())
    }
}
