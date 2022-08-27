// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on https://github.com/Lokathor/wide (Zlib)

use bytemuck::cast;

#[cfg(all(feature = "simd", target_arch = "x86"))]
use core::arch::x86::*;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use core::arch::x86_64::*;

use crate::wide::{f32x8, u32x8};

cfg_if::cfg_if! {
    if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct i32x8(__m256i);
    } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct i32x8(pub __m128i, pub __m128i);
    } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
        use core::arch::wasm32::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct i32x8(pub v128, pub v128);
    } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
        use core::arch::aarch64::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct i32x8(pub int32x4_t, pub int32x4_t);
    } else {
        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct i32x8([i32; 8]);
    }
}

unsafe impl bytemuck::Zeroable for i32x8 {}
unsafe impl bytemuck::Pod for i32x8 {}

impl Default for i32x8 {
    fn default() -> Self {
        Self::splat(0)
    }
}

impl i32x8 {
    pub fn splat(n: i32) -> Self {
        cast([n, n, n, n, n, n, n, n])
    }

    pub fn blend(self, t: Self, f: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_blendv_epi8(f.0, t.0, self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                Self(
                    unsafe { _mm_blendv_epi8(f.0, t.0, self.0) },
                    unsafe { _mm_blendv_epi8(f.1, t.1, self.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_bitselect(t.0, f.0, self.0), v128_bitselect(t.1, f.1, self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(
                    unsafe { vbslq_s32(cast(self.0), t.0, f.0) },
                    unsafe { vbslq_s32(cast(self.1), t.1, f.1) },
                )
            } else {
                super::generic_bit_blend(self, t, f)
            }
        }
    }

    pub fn cmp_eq(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_cmpeq_epi32(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                cast(Self(
                    cast(unsafe { _mm_cmpeq_epi32(self.0, rhs.0) }),
                    cast(unsafe { _mm_cmpeq_epi32(self.1, rhs.1) }),
                ))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(i32x4_eq(self.0, rhs.0), i32x4_eq(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(
                    unsafe { cast(vceqq_s32(self.0, rhs.0)) },
                    unsafe { cast(vceqq_s32(self.1, rhs.1)) },
                )
            } else {
                Self(impl_x8_cmp!(self, eq, rhs, -1, 0))
            }
        }
    }

    pub fn cmp_gt(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_cmpgt_epi32(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                cast(Self(
                    cast(unsafe { _mm_cmpgt_epi32(self.0, rhs.0) }),
                    cast(unsafe { _mm_cmpgt_epi32(self.1, rhs.1) }),
                ))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(i32x4_gt(self.0, rhs.0), i32x4_eq(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(
                    unsafe { cast(vcgtq_s32(self.0, rhs.0)) },
                    unsafe { cast(vcgtq_s32(self.1, rhs.1)) },
                )
            } else {
                Self(impl_x8_cmp!(self, gt, rhs, -1, 0))
            }
        }
    }

    pub fn cmp_lt(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                // There is no `_mm256_cmpLT_epi32`, therefore we have to use
                // `_mm256_cmpGT_epi32` and then invert the result.
                let v = unsafe { _mm256_cmpgt_epi32(self.0, rhs.0) };
                let all_bits = unsafe { _mm256_set1_epi16(-1) };
                Self(unsafe { _mm256_xor_si256(v, all_bits) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                cast(Self(
                    cast(unsafe { _mm_cmplt_epi32(self.0, rhs.0) }),
                    cast(unsafe { _mm_cmplt_epi32(self.1, rhs.1) }),
                ))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(i32x4_lt(self.0, rhs.0), i32x4_lt(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(
                    unsafe { cast(vcltq_s32(self.0, rhs.0)) },
                    unsafe { cast(vcltq_s32(self.1, rhs.1)) },
                )
            } else {
                Self(impl_x8_cmp!(self, lt, rhs, -1, 0))
            }
        }
    }

    pub fn to_f32x8(self) -> f32x8 {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                cast(unsafe { _mm256_cvtepi32_ps(self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                cast(Self(
                    cast(unsafe { _mm_cvtepi32_ps(self.0) }),
                    cast(unsafe { _mm_cvtepi32_ps(self.1) }),
                ))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                cast(Self(f32x4_convert_i32x4(self.0), f32x4_convert_i32x4(self.1)))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                cast(Self(
                    unsafe { cast(vcvtq_f32_s32(self.0)) },
                    unsafe { cast(vcvtq_f32_s32(self.1)) },
                ))
            } else {
                let arr: [i32; 8] = cast(self);
                cast([
                    arr[0] as f32,
                    arr[1] as f32,
                    arr[2] as f32,
                    arr[3] as f32,
                    arr[4] as f32,
                    arr[5] as f32,
                    arr[6] as f32,
                    arr[7] as f32,
                ])
            }
        }
    }

    pub fn to_u32x8_bitcast(self) -> u32x8 {
        bytemuck::cast(self)
    }

    pub fn to_f32x8_bitcast(self) -> f32x8 {
        bytemuck::cast(self)
    }
}

impl From<[i32; 8]> for i32x8 {
    fn from(v: [i32; 8]) -> Self {
        cast(v)
    }
}

impl From<i32x8> for [i32; 8] {
    fn from(v: i32x8) -> Self {
        cast(v)
    }
}

impl core::ops::Add for i32x8 {
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
                Self(i32x4_add(self.0, rhs.0), i32x4_add(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(unsafe { vaddq_s32(self.0, rhs.0) }, unsafe { vaddq_s32(self.1, rhs.1) })
            } else {
                Self(impl_x8_op!(self, wrapping_add, rhs))
            }
        }
    }
}

impl core::ops::BitAnd for i32x8 {
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
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(unsafe { vandq_s32(self.0, rhs.0) }, unsafe { vandq_s32(self.1, rhs.1) })
            } else {
                Self(impl_x8_op!(self, bitand, rhs))
            }
        }
    }
}

impl core::ops::Mul for i32x8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_mullo_epi32(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                Self(
                    unsafe { _mm_mullo_epi32(self.0, rhs.0) },
                    unsafe { _mm_mullo_epi32(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(i32x4_mul(self.0, rhs.0), i32x4_mul(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(unsafe { vmulq_s32(self.0, rhs.0) }, unsafe { vmulq_s32(self.1, rhs.1) })
            } else {
                struct Dummy([i32; 8]);
                let arr1: [i32; 8] = cast(self);
                let arr2: [i32; 8] = cast(rhs);
                cast(impl_x8_op!(Dummy(arr1), wrapping_mul, Dummy(arr2)))
            }
        }
    }
}

impl core::ops::BitOr for i32x8 {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_or_si256(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_or_si128(self.0, rhs.0) },
                    unsafe { _mm_or_si128(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_or(self.0, rhs.0), v128_or(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(unsafe { vorrq_s32(self.0, rhs.0) }, unsafe { vorrq_s32(self.1, rhs.1) })
            } else {
                Self(impl_x8_op!(self, bitor, rhs))
            }
        }
    }
}

impl core::ops::BitXor for i32x8 {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx2"))] {
                Self(unsafe { _mm256_xor_si256(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_xor_si128(self.0, rhs.0) },
                    unsafe { _mm_xor_si128(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_xor(self.0, rhs.0), v128_xor(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                Self(unsafe { veorq_s32(self.0, rhs.0) }, unsafe { veorq_s32(self.1, rhs.1) })
            } else {
                Self(impl_x8_op!(self, bitxor, rhs))
            }
        }
    }
}
