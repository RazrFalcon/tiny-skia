// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on https://github.com/Lokathor/wide (Zlib)

use bytemuck::cast;
#[cfg(feature = "simd")] use safe_arch::*;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "simd", target_feature = "sse"))] {
        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        #[repr(C, align(16))]
        pub struct f32x4(m128);
    } else {
        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        #[repr(C, align(16))]
        pub struct f32x4([f32; 4]);
    }
}

unsafe impl bytemuck::Zeroable for f32x4 {}
unsafe impl bytemuck::Pod for f32x4 {}

impl f32x4 {
    pub fn splat(n: f32) -> Self {
        Self::from([n, n, n, n])
    }

    #[inline]
    pub fn max(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse"))] {
                Self(max_m128(self.0, rhs.0))
            } else {
                Self([
                    self.0[0].max(rhs.0[0]),
                    self.0[1].max(rhs.0[1]),
                    self.0[2].max(rhs.0[2]),
                    self.0[3].max(rhs.0[3]),
                ])
            }
        }
    }

    #[inline]
    pub fn min(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse"))] {
                Self(min_m128(self.0, rhs.0))
            } else {
                Self([
                    self.0[0].min(rhs.0[0]),
                    self.0[1].min(rhs.0[1]),
                    self.0[2].min(rhs.0[2]),
                    self.0[3].min(rhs.0[3]),
                ])
            }
        }
    }
}

impl From<[f32; 4]> for f32x4 {
    fn from(v: [f32; 4]) -> Self {
        cast(v)
    }
}

impl From<f32x4> for [f32; 4] {
    fn from(v: f32x4) -> Self {
        cast(v)
    }
}

impl std::ops::Add for f32x4 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse"))] {
                Self(add_m128(self.0, rhs.0))
            } else {
                Self([
                    self.0[0] + rhs.0[0],
                    self.0[1] + rhs.0[1],
                    self.0[2] + rhs.0[2],
                    self.0[3] + rhs.0[3],
                ])
            }
        }
    }
}

impl std::ops::AddAssign for f32x4 {
    #[inline]
    fn add_assign(&mut self, rhs: f32x4) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for f32x4 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse"))] {
                Self(sub_m128(self.0, rhs.0))
            } else {
                Self([
                    self.0[0] - rhs.0[0],
                    self.0[1] - rhs.0[1],
                    self.0[2] - rhs.0[2],
                    self.0[3] - rhs.0[3],
                ])
            }
        }
    }
}

impl std::ops::Mul for f32x4 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse"))] {
                Self(mul_m128(self.0, rhs.0))
            } else {
                Self([
                    self.0[0] * rhs.0[0],
                    self.0[1] * rhs.0[1],
                    self.0[2] * rhs.0[2],
                    self.0[3] * rhs.0[3],
                ])
            }
        }
    }
}

impl std::ops::MulAssign for f32x4 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32x4) {
        *self = *self * rhs;
    }
}
