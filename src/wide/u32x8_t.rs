// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on https://github.com/Lokathor/wide (Zlib)

use crate::wide::{i32x8, f32x8};

#[cfg(all(feature = "simd", target_arch = "x86"))]
use core::arch::x86::*;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use core::arch::x86_64::*;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
        use bytemuck::cast;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct u32x8(__m256i);
    } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
        use bytemuck::cast;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct u32x8(__m128i, __m128i);
    } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
        use core::arch::wasm32::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct u32x8(v128, v128);
    } else {
        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct u32x8([u32; 8]);
    }
}

unsafe impl bytemuck::Zeroable for u32x8 {}
unsafe impl bytemuck::Pod for u32x8 {}

impl Default for u32x8 {
    fn default() -> Self {
        Self::splat(0)
    }
}

impl u32x8 {
    pub fn splat(n: u32) -> Self {
        bytemuck::cast([n, n, n, n, n, n, n, n])
    }

    pub fn to_i32x8_bitcast(self) -> i32x8 {
        bytemuck::cast(self)
    }

    pub fn to_f32x8_bitcast(self) -> f32x8 {
        bytemuck::cast(self)
    }

    pub fn cmp_eq(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_cmpeq_epi32(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_cmpeq_epi32(self.0, rhs.0) },
                    unsafe { _mm_cmpeq_epi32(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(u32x4_eq(self.0, rhs.0), u32x4_eq(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, eq, rhs, u32::MAX, 0))
            }
        }
    }
}

impl core::ops::Not for u32x8 {
    type Output = Self;

    fn not(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                let all_bits = unsafe { _mm256_set1_epi16(-1) };
                Self(unsafe { _mm256_xor_si256(self.0, all_bits) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                let all_bits = unsafe { _mm_set1_epi32(-1) };
                Self(
                    unsafe { _mm_xor_si128(self.0, all_bits) },
                    unsafe { _mm_xor_si128(self.1, all_bits) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_not(self.0), v128_not(self.1))
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

impl core::ops::Add for u32x8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_add_epi32(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_add_epi32(self.0, rhs.0) },
                    unsafe { _mm_add_epi32(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(u32x4_add(self.0, rhs.0), u32x4_add(self.1, rhs.1))
            } else {
                Self(impl_x8_op!(self, wrapping_add, rhs))
            }
        }
    }
}

impl core::ops::BitAnd for u32x8 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_and_si256(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_and_si128(self.0, rhs.0) },
                    unsafe { _mm_and_si128(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_and(self.0, rhs.0), v128_and(self.1, rhs.1))
            } else {
                Self(impl_x8_op!(self, bitand, rhs))
            }
        }
    }
}

impl core::ops::Shl<i32> for u32x8 {
    type Output = Self;

    fn shl(self, rhs: i32) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                let shift: __m128i = cast([rhs as u64, 0]);
                Self(unsafe { _mm256_sll_epi32(self.0, shift) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                let shift = cast([rhs as u64, 0]);
                Self(
                    unsafe { _mm_sll_epi32(self.0, shift) },
                    unsafe { _mm_sll_epi32(self.1, shift) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(u32x4_shl(self.0, rhs as _), u32x4_shl(self.1, rhs as _))
            } else {
                let u = rhs as u64;
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

impl core::ops::Shr<i32> for u32x8 {
    type Output = Self;

    fn shr(self, rhs: i32) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                let shift: __m128i = cast([rhs as u64, 0]);
                Self(unsafe { _mm256_srl_epi32(self.0, shift) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                let shift: __m128i = cast([rhs as u64, 0]);
                Self(
                    unsafe { _mm_srl_epi32(self.0, shift) },
                    unsafe { _mm_srl_epi32(self.1, shift) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(u32x4_shr(self.0, rhs as _), u32x4_shr(self.1, rhs as _))
            } else {
                let u = rhs as u64;
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
