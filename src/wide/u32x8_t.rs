// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on https://github.com/Lokathor/wide (Zlib)

#[cfg(feature = "simd")] use bytemuck::cast;
#[cfg(feature = "simd")] use safe_arch::*;

use crate::wide::{i32x8, f32x8};

cfg_if::cfg_if! {
    if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(C, align(32))]
        pub struct u32x8(m256i);
    } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(C, align(32))]
        pub struct u32x8(m128i, m128i);
    } else {
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(C, align(32))]
        pub struct u32x8([u32; 8]);
    }
}

unsafe impl bytemuck::Zeroable for u32x8 {}
unsafe impl bytemuck::Pod for u32x8 {}

impl u32x8 {
    pub fn splat(n: u32) -> Self {
        bytemuck::cast([n, n, n, n, n, n, n, n])
    }

    #[inline]
    pub fn to_i32x8_bitcast(self) -> i32x8 {
        bytemuck::cast(self)
    }

    #[inline]
    pub fn to_f32x8_bitcast(self) -> f32x8 {
        bytemuck::cast(self)
    }

    #[inline]
    pub fn cmp_eq(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(cmp_eq_mask_i32_m256i(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(cmp_eq_mask_i32_m128i(self.0, rhs.0), cmp_eq_mask_i32_m128i(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, eq, rhs, u32::MAX, 0))
            }
        }
    }
}

impl std::ops::Not for u32x8 {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(self.0.not())
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(self.0.not(), self.1.not())
            } else {
                Self([
                    !self.0[0],
                    !self.0[1],
                    !self.0[2],
                    !self.0[3],
                    !self.0[4],
                    !self.0[5],
                    !self.0[6],
                    !self.0[7],
                ])
            }
        }
    }
}

impl std::ops::Add for u32x8 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(add_i32_m256i(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(add_i32_m128i(self.0, rhs.0), add_i32_m128i(self.1, rhs.1))
            } else {
                Self(impl_x8_op!(self, wrapping_add, rhs))
            }
        }
    }
}

impl std::ops::BitAnd for u32x8 {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(bitand_m256i(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(bitand_m128i(self.0, rhs.0), bitand_m128i(self.1, rhs.1))
            } else {
                Self(impl_x8_op!(self, bitand, rhs))
            }
        }
    }
}

impl std::ops::Shl<i32> for u32x8 {
    type Output = Self;

    #[inline]
    fn shl(self, rhs: i32) -> Self::Output {
        let u = rhs as u64;
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                let shift = cast([u, 0]);
                Self(shl_all_u32_m256i(self.0, shift))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                let shift = cast([u, 0]);
                Self(shl_all_u32_m128i(self.0, shift), shl_all_u32_m128i(self.1, shift))
            } else {
                Self([
                    self.0[0] << u,
                    self.0[1] << u,
                    self.0[2] << u,
                    self.0[3] << u,
                    self.0[4] << u,
                    self.0[5] << u,
                    self.0[6] << u,
                    self.0[7] << u,
                ])
            }
        }
    }
}

impl std::ops::Shr<i32> for u32x8 {
    type Output = Self;

    #[inline]
    fn shr(self, rhs: i32) -> Self::Output {
        let u = rhs as u64;
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                let shift = cast([u, 0]);
                Self(shr_all_u32_m256i(self.0, shift))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                let shift = cast([u, 0]);
                Self(shr_all_u32_m128i(self.0, shift), shr_all_u32_m128i(self.1, shift))
            } else {
                Self([
                    self.0[0] >> u,
                    self.0[1] >> u,
                    self.0[2] >> u,
                    self.0[3] >> u,
                    self.0[4] >> u,
                    self.0[5] >> u,
                    self.0[6] >> u,
                    self.0[7] >> u,
                ])
            }
        }
    }
}
