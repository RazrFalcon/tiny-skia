// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on https://github.com/Lokathor/wide (Zlib)

use bytemuck::cast;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(16))]
        pub struct f32x4(__m128);
    } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
        use core::arch::wasm32::*;

        // repr(transparent) allows for directly passing the v128 on the WASM stack.
        #[derive(Clone, Copy, Debug)]
        #[repr(transparent)]
        pub struct f32x4(v128);
    } else {
        #[derive(Clone, Copy, Debug)]
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

    pub fn max(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(unsafe { _mm_max_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_max(self.0, rhs.0))
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

    pub fn min(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(unsafe { _mm_min_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_min(self.0, rhs.0))
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

impl core::ops::Add for f32x4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(unsafe { _mm_add_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_add(self.0, rhs.0))
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

impl core::ops::AddAssign for f32x4 {
    fn add_assign(&mut self, rhs: f32x4) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for f32x4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(unsafe { _mm_sub_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_sub(self.0, rhs.0))
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

impl core::ops::Mul for f32x4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(unsafe { _mm_mul_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_mul(self.0, rhs.0))
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

impl core::ops::MulAssign for f32x4 {
    fn mul_assign(&mut self, rhs: f32x4) {
        *self = *self * rhs;
    }
}
