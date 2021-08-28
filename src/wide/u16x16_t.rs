// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// No need to use explicit 256bit AVX2 SIMD.
// `-C target-cpu=native` will autovectorize it better than us.
// Not even sure why explicit instructions are so slow...
//
// We also have to inline all the methods. They are pretty large,
// but without the inlining the performance is plummeting.

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct u16x16(pub [u16; 16]);

macro_rules! impl_u16x16_op {
    ($a:expr, $op:ident, $b:expr) => {
        u16x16([
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

impl u16x16 {
    #[inline]
    pub fn splat(n: u16) -> Self {
        u16x16([n, n, n, n, n, n, n, n, n, n, n, n, n, n, n, n])
    }

    #[inline]
    pub fn as_slice(&self) -> &[u16; 16] {
        &self.0
    }

    #[inline]
    pub fn min(&self, other: &Self) -> Self {
        impl_u16x16_op!(self, min, other)
    }

    #[inline]
    pub fn max(&self, other: &Self) -> Self {
        impl_u16x16_op!(self, max, other)
    }

    #[inline]
    pub fn cmp_le(&self, other: &Self) -> Self {
        u16x16([
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

    #[inline]
    pub fn blend(self, t: Self, e: Self) -> Self {
        (t & self) | (e & !self)
    }
}

impl core::ops::Add<u16x16> for u16x16 {
    type Output = Self;

    #[inline]
    fn add(self, other: u16x16) -> Self::Output {
        impl_u16x16_op!(self, add, other)
    }
}

impl core::ops::Sub<u16x16> for u16x16 {
    type Output = Self;

    #[inline]
    fn sub(self, other: u16x16) -> Self::Output {
        impl_u16x16_op!(self, sub, other)
    }
}

impl core::ops::Mul<u16x16> for u16x16 {
    type Output = Self;

    #[inline]
    fn mul(self, other: u16x16) -> Self::Output {
        impl_u16x16_op!(self, mul, other)
    }
}

impl core::ops::Div<u16x16> for u16x16 {
    type Output = Self;

    #[inline]
    fn div(self, other: u16x16) -> Self::Output {
        impl_u16x16_op!(self, div, other)
    }
}

impl core::ops::BitAnd<u16x16> for u16x16 {
    type Output = Self;

    #[inline]
    fn bitand(self, other: u16x16) -> Self::Output {
        impl_u16x16_op!(self, bitand, other)
    }
}

impl core::ops::BitOr<u16x16> for u16x16 {
    type Output = Self;

    #[inline]
    fn bitor(self, other: u16x16) -> Self::Output {
        impl_u16x16_op!(self, bitor, other)
    }
}

impl core::ops::Not for u16x16 {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        u16x16([
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
