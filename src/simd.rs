// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Loosely based on `pathfinder_simd` (Apache 2.0/MIT).

#[cfg(all(feature = "sse2", any(target_arch = "x86", target_arch = "x86_64")))]
mod x86 {
    #[cfg(target_pointer_width = "32")]
    use std::arch::x86::{__m128, __m128i};
    #[cfg(target_pointer_width = "32")]
    use std::arch::x86;
    #[cfg(target_pointer_width = "64")]
    use std::arch::x86_64::{__m128, __m128i};
    #[cfg(target_pointer_width = "64")]
    use std::arch::x86_64 as x86;


    #[derive(Copy, Clone)]
    pub struct I32x4(__m128i);

    impl I32x4 {
        #[inline(always)]
        pub fn as_slice(&self) -> &[i32; 4] {
            unsafe { &*(&self.0 as *const __m128i as *const [i32; 4]) }
        }

        #[inline(always)] pub fn x(&self) -> i32 { self.as_slice()[0] }
        #[inline(always)] pub fn y(&self) -> i32 { self.as_slice()[1] }
        #[inline(always)] pub fn z(&self) -> i32 { self.as_slice()[2] }
        #[inline(always)] pub fn w(&self) -> i32 { self.as_slice()[3] }
    }

    impl std::fmt::Debug for I32x4 {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            write!(f, "I32x4({:?})", self.as_slice())
        }
    }


    #[derive(Copy, Clone)]
    pub struct U32x4(__m128i);

    impl U32x4 {
        #[inline(always)]
        pub fn as_slice(&self) -> &[u32; 4] {
            unsafe { &*(&self.0 as *const __m128i as *const [u32; 4]) }
        }

        /// Returns true if all four booleans in this vector are true.
        ///
        /// The result is *undefined* if all four values in this vector are not booleans. A boolean is
        /// a value with all bits set or all bits clear (i.e. !0 or 0).
        #[inline(always)]
        fn all_true(self) -> bool {
            unsafe { x86::_mm_movemask_ps(x86::_mm_castsi128_ps(self.0)) == 0x0f }
        }

        #[inline(always)]
        pub fn if_then_else(&self, t: F32x4, e: F32x4) -> F32x4 {
            unsafe {
                // Do not use _mm_cvtepi32_ps here. We need `self` as bits, no as float.
                let c = *(&self.0 as *const __m128i as *const __m128);
                F32x4(x86::_mm_or_ps(x86::_mm_and_ps(c, t.0), x86::_mm_andnot_ps(c, e.0)))
            }
        }
    }

    impl std::fmt::Debug for U32x4 {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            write!(f, "U32x4({:?})", self.as_slice())
        }
    }


    // Unlike U16x16, F32x4 SIMD is 3x faster than a scalar version.
    #[derive(Copy, Clone)]
    pub struct F32x4(__m128);

    impl F32x4 {
        #[inline(always)]
        pub fn new(a: f32, b: f32, c: f32, d: f32) -> F32x4 {
            unsafe {
                let vector = [a, b, c, d];
                F32x4(x86::_mm_loadu_ps(vector.as_ptr()))
            }
        }

        #[inline(always)]
        pub fn splat(x: f32) -> F32x4 {
            unsafe { F32x4(x86::_mm_set1_ps(x)) }
        }

        #[inline(always)]
        pub fn approx_recip(self) -> F32x4 {
            unsafe { F32x4(x86::_mm_rcp_ps(self.0)) }
        }

        #[inline(always)]
        pub fn approx_recip_sqrt(self) -> F32x4 {
            unsafe { F32x4(x86::_mm_rsqrt_ps(self.0)) }
        }

        #[inline(always)]
        pub fn min(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_min_ps(self.0, other.0)) }
        }

        #[inline(always)]
        pub fn max(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_max_ps(self.0, other.0)) }
        }

        #[inline(always)]
        pub fn to_i32x4(self) -> I32x4 {
            unsafe { I32x4(x86::_mm_cvtps_epi32(self.0)) }
        }

        #[inline(always)]
        pub fn packed_eq(self, other: F32x4) -> U32x4 {
            unsafe {
                U32x4(x86::_mm_castps_si128(x86::_mm_cmpeq_ps(self.0, other.0)))
            }
        }

        #[inline(always)]
        pub fn packed_gt(self, other: F32x4) -> U32x4 {
            unsafe {
                U32x4(x86::_mm_castps_si128(x86::_mm_cmpgt_ps(self.0, other.0)))
            }
        }

        #[inline(always)]
        pub fn packed_ge(self, other: F32x4) -> U32x4 {
            unsafe {
                U32x4(x86::_mm_castps_si128(x86::_mm_cmpge_ps(self.0, other.0)))
            }
        }

        #[inline(always)]
        pub fn packed_le(self, other: F32x4) -> U32x4 {
            unsafe {
                U32x4(x86::_mm_castps_si128(x86::_mm_cmple_ps(self.0, other.0)))
            }
        }

        #[inline(always)]
        pub fn as_slice(&self) -> &[f32; 4] {
            unsafe { &*(&self.0 as *const __m128 as *const [f32; 4]) }
        }

        #[inline(always)] pub fn x(&self) -> f32 { self.as_slice()[0] }
        #[inline(always)] pub fn y(&self) -> f32 { self.as_slice()[1] }
        #[inline(always)] pub fn z(&self) -> f32 { self.as_slice()[2] }
        #[inline(always)] pub fn w(&self) -> f32 { self.as_slice()[3] }
    }

    impl Default for F32x4 {
        #[inline(always)]
        fn default() -> F32x4 {
            unsafe { F32x4(x86::_mm_setzero_ps()) }
        }
    }

    impl std::fmt::Debug for F32x4 {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            write!(f, "F32x4({:?})", self.as_slice())
        }
    }

    impl PartialEq for F32x4 {
        #[inline(always)]
        fn eq(&self, other: &F32x4) -> bool {
            self.packed_eq(*other).all_true()
        }
    }

    impl std::ops::Add<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn add(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_add_ps(self.0, other.0)) }
        }
    }

    impl std::ops::Sub<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn sub(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_sub_ps(self.0, other.0)) }
        }
    }

    impl std::ops::Mul<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn mul(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_mul_ps(self.0, other.0)) }
        }
    }

    impl std::ops::MulAssign for F32x4 {
        #[inline(always)]
        fn mul_assign(&mut self, other: F32x4) {
            *self = *self * other
        }
    }

    impl std::ops::Div<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn div(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_div_ps(self.0, other.0)) }
        }
    }
}

#[cfg(not(feature = "sse2"))]
mod scalar {
    #[derive(Copy, Clone, Default, PartialEq, Debug)]
    pub struct I32x4([i32; 4]);

    impl I32x4 {
        #[inline(always)]
        pub fn as_slice(&self) -> &[i32; 4] {
            &self.0
        }

        #[inline(always)] pub fn x(&self) -> i32 { self.0[0] }
        #[inline(always)] pub fn y(&self) -> i32 { self.0[1] }
        #[inline(always)] pub fn z(&self) -> i32 { self.0[2] }
        #[inline(always)] pub fn w(&self) -> i32 { self.0[3] }
    }


    #[derive(Copy, Clone, Default, PartialEq, Debug)]
    pub struct U32x4([u32; 4]);

    impl U32x4 {
        #[inline(always)]
        pub fn if_then_else(&self, t: F32x4, e: F32x4) -> F32x4 {
            F32x4([
                if self.x() != 0 { t.x() } else { e.x() },
                if self.y() != 0 { t.y() } else { e.y() },
                if self.z() != 0 { t.z() } else { e.z() },
                if self.w() != 0 { t.w() } else { e.w() },
            ])
        }

        #[inline(always)]
        pub fn as_slice(&self) -> &[u32; 4] {
            &self.0
        }

        #[inline(always)] pub fn x(&self) -> u32 { self.as_slice()[0] }
        #[inline(always)] pub fn y(&self) -> u32 { self.as_slice()[1] }
        #[inline(always)] pub fn z(&self) -> u32 { self.as_slice()[2] }
        #[inline(always)] pub fn w(&self) -> u32 { self.as_slice()[3] }
    }


    #[derive(Copy, Clone, Default, PartialEq, Debug)]
    pub struct F32x4([f32; 4]);

    impl F32x4 {
        #[inline(always)]
        pub fn new(a: f32, b: f32, c: f32, d: f32) -> F32x4 {
            F32x4([a, b, c, d])
        }

        #[inline(always)]
        pub fn splat(x: f32) -> F32x4 {
            F32x4([x; 4])
        }

        #[inline(always)]
        pub fn approx_recip(self) -> F32x4 {
            F32x4([
                1.0 / self.0[0],
                1.0 / self.0[1],
                1.0 / self.0[2],
                1.0 / self.0[3],
            ])
        }

        #[inline(always)]
        pub fn approx_recip_sqrt(self) -> F32x4 {
            F32x4([
                1.0 / self.0[0].sqrt(),
                1.0 / self.0[1].sqrt(),
                1.0 / self.0[2].sqrt(),
                1.0 / self.0[3].sqrt(),
            ])
        }

        #[inline(always)]
        pub fn min(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x().min(other.x()),
                self.y().min(other.y()),
                self.z().min(other.z()),
                self.w().min(other.w()),
            ])
        }

        #[inline(always)]
        pub fn max(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x().max(other.x()),
                self.y().max(other.y()),
                self.z().max(other.z()),
                self.w().max(other.w()),
            ])
        }

        #[inline(always)]
        pub fn to_i32x4(&self) -> I32x4 {
            I32x4([
                self.x().round() as i32,
                self.y().round() as i32,
                self.z().round() as i32,
                self.w().round() as i32,
            ])
        }

        #[inline(always)]
        pub fn packed_eq(self, other: F32x4) -> U32x4 {
            U32x4([
                if self.0[0] == other.0[0] { !0 } else { 0 },
                if self.0[1] == other.0[1] { !0 } else { 0 },
                if self.0[2] == other.0[2] { !0 } else { 0 },
                if self.0[3] == other.0[3] { !0 } else { 0 },
            ])
        }

        #[inline(always)]
        pub fn packed_gt(self, other: F32x4) -> U32x4 {
            U32x4([
                if self.0[0] > other.0[0] { !0 } else { 0 },
                if self.0[1] > other.0[1] { !0 } else { 0 },
                if self.0[2] > other.0[2] { !0 } else { 0 },
                if self.0[3] > other.0[3] { !0 } else { 0 },
            ])
        }

        #[inline(always)]
        pub fn packed_ge(self, other: F32x4) -> U32x4 {
            U32x4([
                if self.0[0] >= other.0[0] { !0 } else { 0 },
                if self.0[1] >= other.0[1] { !0 } else { 0 },
                if self.0[2] >= other.0[2] { !0 } else { 0 },
                if self.0[3] >= other.0[3] { !0 } else { 0 },
            ])
        }

        #[inline(always)]
        pub fn packed_le(self, other: F32x4) -> U32x4 {
            U32x4([
                if self.0[0] <= other.0[0] { !0 } else { 0 },
                if self.0[1] <= other.0[1] { !0 } else { 0 },
                if self.0[2] <= other.0[2] { !0 } else { 0 },
                if self.0[3] <= other.0[3] { !0 } else { 0 },
            ])
        }

        #[allow(dead_code)]
        #[inline(always)]
        pub fn as_slice(&self) -> &[f32; 4] { &self.0 }

        #[inline(always)] pub fn x(&self) -> f32 { self.0[0] }
        #[inline(always)] pub fn y(&self) -> f32 { self.0[1] }
        #[inline(always)] pub fn z(&self) -> f32 { self.0[2] }
        #[inline(always)] pub fn w(&self) -> f32 { self.0[3] }
    }

    impl std::ops::Add<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn add(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() + other.x(),
                self.y() + other.y(),
                self.z() + other.z(),
                self.w() + other.w(),
            ])
        }
    }

    impl std::ops::Sub<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn sub(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() - other.x(),
                self.y() - other.y(),
                self.z() - other.z(),
                self.w() - other.w(),
            ])
        }
    }

    impl std::ops::Mul<F32x4> for F32x4 {
        type Output = F32x4;
        #[inline(always)]
        fn mul(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() * other.x(),
                self.y() * other.y(),
                self.z() * other.z(),
                self.w() * other.w(),
            ])
        }
    }

    impl std::ops::MulAssign for F32x4 {
        #[inline(always)]
        fn mul_assign(&mut self, other: F32x4) {
            *self = *self * other
        }
    }

    impl std::ops::Div<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn div(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() / other.x(),
                self.y() / other.y(),
                self.z() / other.z(),
                self.w() / other.w(),
            ])
        }
    }
}

#[cfg(all(feature = "sse2", any(target_arch = "x86", target_arch = "x86_64")))]
pub use x86::{I32x4, U32x4, F32x4};

#[cfg(not(feature = "sse2"))]
pub use scalar::{I32x4, U32x4, F32x4};


// Right now, there are no visible benefits of using SIMD for F32x2. So we don't.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct F32x2([f32; 2]);

impl F32x2 {
    #[inline(always)]
    pub fn new(a: f32, b: f32) -> F32x2 {
        F32x2([a, b])
    }

    #[inline(always)]
    pub fn splat(x: f32) -> F32x2 {
        F32x2([x, x])
    }

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.0[1]
    }
}

impl std::ops::Add<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn add(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() + other.x(),
            self.y() + other.y(),
        ])
    }
}

impl std::ops::Sub<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn sub(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() - other.x(),
            self.y() - other.y(),
        ])
    }
}

impl std::ops::Mul<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn mul(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() * other.x(),
            self.y() * other.y(),
        ])
    }
}

impl std::ops::Div<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn div(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() / other.x(),
            self.y() / other.y(),
        ])
    }
}


// No need to use explicit 256bit AVX2 SIMD.
// With `-C target-cpu=native` it will autovectorize it better than us.
// Not even sure why explicit instructions are so slow...
#[derive(Copy, Clone, Default, Debug)]
pub struct U16x16([u16; 16]);

macro_rules! impl_u16x16_op {
    ($a:expr, $op:ident, $b:expr) => {
        U16x16([
            $a.0[ 0].$op($b.0[ 0]),
            $a.0[ 1].$op($b.0[ 1]),
            $a.0[ 2].$op($b.0[ 2]),
            $a.0[ 3].$op($b.0[ 3]),
            $a.0[ 4].$op($b.0[ 4]),
            $a.0[ 5].$op($b.0[ 5]),
            $a.0[ 6].$op($b.0[ 6]),
            $a.0[ 7].$op($b.0[ 7]),
            $a.0[ 8].$op($b.0[ 8]),
            $a.0[ 9].$op($b.0[ 9]),
            $a.0[10].$op($b.0[10]),
            $a.0[11].$op($b.0[11]),
            $a.0[12].$op($b.0[12]),
            $a.0[13].$op($b.0[13]),
            $a.0[14].$op($b.0[14]),
            $a.0[15].$op($b.0[15]),
        ])
    };
}

impl U16x16 {
    #[inline(always)]
    pub fn new(n: [u16; 16]) -> Self {
        U16x16(n)
    }

    #[inline(always)]
    pub fn splat(n: u16) -> Self {
        U16x16([n, n, n, n, n, n, n, n, n, n, n, n, n, n, n, n])
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u16; 16] {
        &self.0
    }

    #[inline(always)]
    pub fn min(&self, other: &Self) -> Self {
        impl_u16x16_op!(self, min, other)
    }

    #[inline(always)]
    pub fn max(&self, other: &Self) -> Self {
        impl_u16x16_op!(self, max, other)
    }

    #[inline(always)]
    pub fn packed_le(self, other: Self) -> Self {
        U16x16([
            if self.0[ 0] <= other.0[ 0] { !0 } else { 0 },
            if self.0[ 1] <= other.0[ 1] { !0 } else { 0 },
            if self.0[ 2] <= other.0[ 2] { !0 } else { 0 },
            if self.0[ 3] <= other.0[ 3] { !0 } else { 0 },
            if self.0[ 4] <= other.0[ 4] { !0 } else { 0 },
            if self.0[ 5] <= other.0[ 5] { !0 } else { 0 },
            if self.0[ 6] <= other.0[ 6] { !0 } else { 0 },
            if self.0[ 7] <= other.0[ 7] { !0 } else { 0 },
            if self.0[ 8] <= other.0[ 8] { !0 } else { 0 },
            if self.0[ 9] <= other.0[ 9] { !0 } else { 0 },
            if self.0[10] <= other.0[10] { !0 } else { 0 },
            if self.0[11] <= other.0[11] { !0 } else { 0 },
            if self.0[12] <= other.0[12] { !0 } else { 0 },
            if self.0[13] <= other.0[13] { !0 } else { 0 },
            if self.0[14] <= other.0[14] { !0 } else { 0 },
            if self.0[15] <= other.0[15] { !0 } else { 0 },
        ])
    }

    #[inline(always)]
    pub fn if_then_else(self, t: Self, e: Self) -> Self {
        (t & self) | (e & !self)
    }
}

impl std::ops::Add<U16x16> for U16x16 {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: U16x16) -> Self::Output {
        impl_u16x16_op!(self, add, other)
    }
}

impl std::ops::Sub<U16x16> for U16x16 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: U16x16) -> Self::Output {
        impl_u16x16_op!(self, sub, other)
    }
}

impl std::ops::Mul<U16x16> for U16x16 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: U16x16) -> Self::Output {
        impl_u16x16_op!(self, mul, other)
    }
}

impl std::ops::Div<U16x16> for U16x16 {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: U16x16) -> Self::Output {
        impl_u16x16_op!(self, div, other)
    }
}

impl std::ops::BitAnd<U16x16> for U16x16 {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, other: U16x16) -> Self::Output {
        impl_u16x16_op!(self, bitand, other)
    }
}

impl std::ops::BitOr<U16x16> for U16x16 {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, other: U16x16) -> Self::Output {
        impl_u16x16_op!(self, bitor, other)
    }
}

impl std::ops::Not for U16x16 {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        U16x16([
            !self.0[ 0],
            !self.0[ 1],
            !self.0[ 2],
            !self.0[ 3],
            !self.0[ 4],
            !self.0[ 5],
            !self.0[ 6],
            !self.0[ 7],
            !self.0[ 8],
            !self.0[ 9],
            !self.0[10],
            !self.0[11],
            !self.0[12],
            !self.0[13],
            !self.0[14],
            !self.0[15],
        ])
    }
}
