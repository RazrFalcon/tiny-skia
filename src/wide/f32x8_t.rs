// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on https://github.com/Lokathor/wide (Zlib)

use bytemuck::cast;

use crate::wide::{u32x8, i32x8};

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use tiny_skia_path::NoStdFloat;

#[cfg(all(feature = "simd", target_arch = "x86"))]
use core::arch::x86::*;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use core::arch::x86_64::*;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "simd", target_feature = "avx"))] {
        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8(__m256);
    } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8(__m128, __m128);
    } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
        use core::arch::wasm32::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8(v128, v128);
    } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
        use core::arch::aarch64::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8(float32x4_t, float32x4_t);
    } else {
        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8([f32; 8]);
    }
}

unsafe impl bytemuck::Zeroable for f32x8 {}
unsafe impl bytemuck::Pod for f32x8 {}

impl Default for f32x8 {
    fn default() -> Self {
        Self::splat(0.0)
    }
}

impl f32x8 {
    pub fn splat(n: f32) -> Self {
        cast([n, n, n, n, n, n, n, n])
    }

    pub fn floor(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_floor(self.0), f32x4_floor(self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vrndmq_f32(self.0), vrndmq_f32(self.1))
                }
            } else {
                let roundtrip: f32x8 = cast(self.trunc_int().to_f32x8());
                roundtrip - roundtrip.cmp_gt(self).blend(f32x8::splat(1.0), f32x8::default())
            }
        }
    }

    pub fn fract(self) -> Self {
        self - self.floor()
    }

    pub fn normalize(self) -> Self {
        self.max(f32x8::default()).min(f32x8::splat(1.0))
    }

    pub fn to_i32x8_bitcast(self) -> i32x8 {
        bytemuck::cast(self)
    }

    pub fn to_u32x8_bitcast(self) -> u32x8 {
        bytemuck::cast(self)
    }

    pub fn cmp_eq(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_EQ_OQ) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_cmpeq_ps(self.0, rhs.0) },
                    unsafe { _mm_cmpeq_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_eq(self.0, rhs.0), f32x4_eq(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vceqq_f32(self.0, rhs.0)),
                        core::mem::transmute(vceqq_f32(self.1, rhs.1)),
                    )
                }
            } else {
                Self(impl_x8_cmp!(self, eq, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_ge(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_GE_OQ) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_cmpge_ps(self.0, rhs.0) },
                    unsafe { _mm_cmpge_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_ge(self.0, rhs.0), f32x4_ge(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vcgeq_f32(self.0, rhs.0)),
                        core::mem::transmute(vcgeq_f32(self.1, rhs.1)),
                    )
                }
            } else {
                Self(impl_x8_cmp!(self, ge, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_gt(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_GT_OQ) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_cmpgt_ps(self.0, rhs.0) },
                    unsafe { _mm_cmpgt_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_gt(self.0, rhs.0), f32x4_gt(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vcgtq_f32(self.0, rhs.0)),
                        core::mem::transmute(vcgtq_f32(self.1, rhs.1)),
                    )
                }
            } else {
                Self(impl_x8_cmp!(self, gt, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_ne(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_NEQ_OQ) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_cmpneq_ps(self.0, rhs.0) },
                    unsafe { _mm_cmpneq_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_ne(self.0, rhs.0), f32x4_ne(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vmvnq_u32(vceqq_f32(self.0, rhs.0))),
                        core::mem::transmute(vmvnq_u32(vceqq_f32(self.1, rhs.1))),
                    )
                }
            } else {
                Self(impl_x8_cmp!(self, ne, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_le(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_LE_OQ) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_cmple_ps(self.0, rhs.0) },
                    unsafe { _mm_cmple_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_le(self.0, rhs.0), f32x4_le(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vcleq_f32(self.0, rhs.0)),
                        core::mem::transmute(vcleq_f32(self.1, rhs.1)),
                    )
                }
            } else {
                Self(impl_x8_cmp!(self, le, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_lt(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_LT_OQ) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_cmplt_ps(self.0, rhs.0) },
                    unsafe { _mm_cmplt_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_lt(self.0, rhs.0), f32x4_lt(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vcltq_f32(self.0, rhs.0)),
                        core::mem::transmute(vcltq_f32(self.1, rhs.1)),
                    )
                }
            } else {
                Self(impl_x8_cmp!(self, lt, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    #[inline]
    pub fn blend(self, t: Self, f: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_blendv_ps(f.0, t.0, self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                Self(
                    unsafe { _mm_blendv_ps(f.0, t.0, self.0) },
                    unsafe { _mm_blendv_ps(f.1, t.1, self.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_bitselect(t.0, f.0, self.0), v128_bitselect(t.1, f.1, self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vbslq_u32(
                            core::mem::transmute(self.0),
                            core::mem::transmute(t.0),
                            core::mem::transmute(f.0),
                        )),
                        core::mem::transmute(vbslq_u32(
                            core::mem::transmute(self.1),
                            core::mem::transmute(t.1),
                            core::mem::transmute(f.1),
                        )),
                    )
                }
            } else {
                super::generic_bit_blend(self, t, f)
            }
        }
    }

    pub fn abs(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_abs(self.0), f32x4_abs(self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vabsq_f32(self.0), vabsq_f32(self.1))
                }
            } else {
                let non_sign_bits = f32x8::splat(f32::from_bits(i32::MAX as u32));
                self & non_sign_bits
            }
        }
    }

    pub fn max(self, rhs: Self) -> Self {
        // These technically don't have the same semantics for NaN and 0, but it
        // doesn't seem to matter as Skia does it the same way.
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_max_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_max_ps(self.0, rhs.0) },
                    unsafe { _mm_max_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_pmax(self.0, rhs.0), f32x4_pmax(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vmaxq_f32(self.0, rhs.0), vmaxq_f32(self.1, rhs.1))
                }
            } else {
                Self([
                    super::pmax(self.0[0], rhs.0[0]),
                    super::pmax(self.0[1], rhs.0[1]),
                    super::pmax(self.0[2], rhs.0[2]),
                    super::pmax(self.0[3], rhs.0[3]),
                    super::pmax(self.0[4], rhs.0[4]),
                    super::pmax(self.0[5], rhs.0[5]),
                    super::pmax(self.0[6], rhs.0[6]),
                    super::pmax(self.0[7], rhs.0[7]),
                ])
            }
        }
    }

    pub fn min(self, rhs: Self) -> Self {
        // These technically don't have the same semantics for NaN and 0, but it
        // doesn't seem to matter as Skia does it the same way.
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_min_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_min_ps(self.0, rhs.0) },
                    unsafe { _mm_min_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_pmin(self.0, rhs.0), f32x4_pmin(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vminq_f32(self.0, rhs.0), vminq_f32(self.1, rhs.1))
                }
            } else {
                Self([
                    super::pmin(self.0[0], rhs.0[0]),
                    super::pmin(self.0[1], rhs.0[1]),
                    super::pmin(self.0[2], rhs.0[2]),
                    super::pmin(self.0[3], rhs.0[3]),
                    super::pmin(self.0[4], rhs.0[4]),
                    super::pmin(self.0[5], rhs.0[5]),
                    super::pmin(self.0[6], rhs.0[6]),
                    super::pmin(self.0[7], rhs.0[7]),
                ])
            }
        }
    }

    pub fn is_finite(self) -> Self {
        let shifted_exp_mask = u32x8::splat(0xFF000000);
        let u: u32x8 = cast(self);
        let shift_u = u.shl::<1>();
        let out = !(shift_u & shifted_exp_mask).cmp_eq(shifted_exp_mask);
        cast(out)
    }

    pub fn round(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_round_ps(self.0, _MM_FROUND_NO_EXC | _MM_FROUND_TO_NEAREST_INT) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                Self(
                    unsafe { _mm_round_ps(self.0, _MM_FROUND_NO_EXC | _MM_FROUND_TO_NEAREST_INT) },
                    unsafe { _mm_round_ps(self.1, _MM_FROUND_NO_EXC | _MM_FROUND_TO_NEAREST_INT) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_nearest(self.0), f32x4_nearest(self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vrndnq_f32(self.0), vrndnq_f32(self.1))
                }
            } else {
                let to_int = f32x8::splat(1.0 / f32::EPSILON);
                let u: u32x8 = cast(self);
                let e: i32x8 = cast(u.shr::<23>() & u32x8::splat(0xff));
                let mut y: f32x8;

                let no_op_magic = i32x8::splat(0x7f + 23);
                let no_op_mask: f32x8 = cast(e.cmp_gt(no_op_magic) | e.cmp_eq(no_op_magic));
                let no_op_val: f32x8 = self;

                let zero_magic = i32x8::splat(0x7f - 1);
                let zero_mask: f32x8 = cast(e.cmp_lt(zero_magic));
                let zero_val: f32x8 = self * f32x8::splat(0.0);

                let neg_bit: f32x8 = cast(cast::<u32x8, i32x8>(u).cmp_lt(i32x8::default()));
                let x: f32x8 = neg_bit.blend(-self, self);
                y = x + to_int - to_int - x;
                y = y.cmp_gt(f32x8::splat(0.5)).blend(
                    y + x - f32x8::splat(-1.0),
                    y.cmp_lt(f32x8::splat(-0.5)).blend(y + x + f32x8::splat(1.0), y + x),
                );
                y = neg_bit.blend(-y, y);

                no_op_mask.blend(no_op_val, zero_mask.blend(zero_val, y))
            }
        }
    }

    pub fn round_int(self) -> i32x8 {
        // These technically don't have the same semantics for NaN and out of
        // range values, but it doesn't seem to matter as Skia does it the same
        // way.
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                cast(unsafe { _mm256_cvtps_epi32(self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                i32x8(
                    unsafe { _mm_cvtps_epi32(self.0) },
                    unsafe { _mm_cvtps_epi32(self.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                let rounded = self.round();
                i32x8(i32x4_trunc_sat_f32x4(rounded.0), i32x4_trunc_sat_f32x4(rounded.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    i32x8(vcvtnq_s32_f32(self.0), vcvtnq_s32_f32(self.1))
                }
            } else {
                let rounded: [f32; 8] = cast(self.round());
                cast([
                    rounded[0] as i32,
                    rounded[1] as i32,
                    rounded[2] as i32,
                    rounded[3] as i32,
                    rounded[4] as i32,
                    rounded[5] as i32,
                    rounded[6] as i32,
                    rounded[7] as i32,
                ])
            }
        }
    }

    pub fn trunc_int(self) -> i32x8 {
        // These technically don't have the same semantics for NaN and out of
        // range values, but it doesn't seem to matter as Skia does it the same
        // way.
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                cast(unsafe { _mm256_cvttps_epi32(self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                i32x8(
                    unsafe { _mm_cvttps_epi32(self.0) },
                    unsafe { _mm_cvttps_epi32(self.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                i32x8(i32x4_trunc_sat_f32x4(self.0), i32x4_trunc_sat_f32x4(self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    i32x8(vcvtq_s32_f32(self.0), vcvtq_s32_f32(self.1))
                }
            } else {
                let n: [f32; 8] = cast(self);
                cast([
                    n[0] as i32,
                    n[1] as i32,
                    n[2] as i32,
                    n[3] as i32,
                    n[4] as i32,
                    n[5] as i32,
                    n[6] as i32,
                    n[7] as i32,
                ])
            }
        }
    }

    pub fn recip_fast(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_rcp_ps(self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_rcp_ps(self.0) },
                    unsafe { _mm_rcp_ps(self.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                let one = f32x4_splat(1.0);
                Self(
                    f32x4_div(one, self.0),
                    f32x4_div(one, self.1),
                )
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    let a = vrecpeq_f32(self.0);
                    let a = vmulq_f32(vrecpsq_f32(self.0, a), a);

                    let b = vrecpeq_f32(self.1);
                    let b = vmulq_f32(vrecpsq_f32(self.1, b), b);

                    Self(a, b)
                }
            } else {
                Self::from([
                    1.0 / self.0[0],
                    1.0 / self.0[1],
                    1.0 / self.0[2],
                    1.0 / self.0[3],
                    1.0 / self.0[4],
                    1.0 / self.0[5],
                    1.0 / self.0[6],
                    1.0 / self.0[7],
                ])
            }
        }
    }

    pub fn recip_sqrt(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_rsqrt_ps(self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_rsqrt_ps(self.0) },
                    unsafe { _mm_rsqrt_ps(self.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                let one = f32x4_splat(1.0);
                Self(
                    f32x4_div(one, f32x4_sqrt(self.0)),
                    f32x4_div(one, f32x4_sqrt(self.1)),
                )
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    let a = vrsqrteq_f32(self.0);
                    let a = vmulq_f32(vrsqrtsq_f32(self.0, vmulq_f32(a, a)), a);

                    let b = vrsqrteq_f32(self.1);
                    let b = vmulq_f32(vrsqrtsq_f32(self.1, vmulq_f32(b, b)), b);

                    Self(a, b)
                }
            } else {
                Self::from([
                    1.0 / self.0[0].sqrt(),
                    1.0 / self.0[1].sqrt(),
                    1.0 / self.0[2].sqrt(),
                    1.0 / self.0[3].sqrt(),
                    1.0 / self.0[4].sqrt(),
                    1.0 / self.0[5].sqrt(),
                    1.0 / self.0[6].sqrt(),
                    1.0 / self.0[7].sqrt(),
                ])
            }
        }
    }

    pub fn sqrt(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_sqrt_ps(self.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_sqrt_ps(self.0) },
                    unsafe { _mm_sqrt_ps(self.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_sqrt(self.0), f32x4_sqrt(self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vsqrtq_f32(self.0), vsqrtq_f32(self.1))
                }
            } else {
                Self::from([
                    self.0[0].sqrt(),
                    self.0[1].sqrt(),
                    self.0[2].sqrt(),
                    self.0[3].sqrt(),
                    self.0[4].sqrt(),
                    self.0[5].sqrt(),
                    self.0[6].sqrt(),
                    self.0[7].sqrt(),
                ])
            }
        }
    }
}

impl From<[f32; 8]> for f32x8 {
    fn from(v: [f32; 8]) -> Self {
        cast(v)
    }
}

impl From<f32x8> for [f32; 8] {
    fn from(v: f32x8) -> Self {
        cast(v)
    }
}

impl core::ops::Add for f32x8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_add_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_add_ps(self.0, rhs.0) },
                    unsafe { _mm_add_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_add(self.0, rhs.0), f32x4_add(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vaddq_f32(self.0, rhs.0), vaddq_f32(self.1, rhs.1))
                }
            } else {
                Self(impl_x8_op!(self, add, rhs))
            }
        }
    }
}

impl core::ops::AddAssign for f32x8 {
    fn add_assign(&mut self, rhs: f32x8) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for f32x8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_sub_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_sub_ps(self.0, rhs.0) },
                    unsafe { _mm_sub_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_sub(self.0, rhs.0), f32x4_sub(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vsubq_f32(self.0, rhs.0), vsubq_f32(self.1, rhs.1))
                }
            } else {
                Self(impl_x8_op!(self, sub, rhs))
            }
        }
    }
}

impl core::ops::Mul for f32x8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_mul_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_mul_ps(self.0, rhs.0) },
                    unsafe { _mm_mul_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_mul(self.0, rhs.0), f32x4_mul(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vmulq_f32(self.0, rhs.0), vmulq_f32(self.1, rhs.1))
                }
            } else {
                Self(impl_x8_op!(self, mul, rhs))
            }
        }
    }
}

impl core::ops::MulAssign for f32x8 {
    fn mul_assign(&mut self, rhs: f32x8) {
        *self = *self * rhs;
    }
}

impl core::ops::Div for f32x8 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_div_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_div_ps(self.0, rhs.0) },
                    unsafe { _mm_div_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_div(self.0, rhs.0), f32x4_div(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(vdivq_f32(self.0, rhs.0), vdivq_f32(self.1, rhs.1))
                }
            } else {
                Self(impl_x8_op!(self, div, rhs))
            }
        }
    }
}

impl core::ops::BitAnd for f32x8 {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_and_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_and_ps(self.0, rhs.0) },
                    unsafe { _mm_and_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_and(self.0, rhs.0), v128_and(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vandq_u32(
                            core::mem::transmute(self.0),
                            core::mem::transmute(rhs.0),
                        )),
                        core::mem::transmute(vandq_u32(
                            core::mem::transmute(self.1),
                            core::mem::transmute(rhs.1),
                        )),
                    )
                }
            } else {
                Self([
                    f32::from_bits(self.0[0].to_bits() & rhs.0[0].to_bits()),
                    f32::from_bits(self.0[1].to_bits() & rhs.0[1].to_bits()),
                    f32::from_bits(self.0[2].to_bits() & rhs.0[2].to_bits()),
                    f32::from_bits(self.0[3].to_bits() & rhs.0[3].to_bits()),
                    f32::from_bits(self.0[4].to_bits() & rhs.0[4].to_bits()),
                    f32::from_bits(self.0[5].to_bits() & rhs.0[5].to_bits()),
                    f32::from_bits(self.0[6].to_bits() & rhs.0[6].to_bits()),
                    f32::from_bits(self.0[7].to_bits() & rhs.0[7].to_bits()),
                ])
            }
        }
    }
}

impl core::ops::BitOr for f32x8 {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_or_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_or_ps(self.0, rhs.0) },
                    unsafe { _mm_or_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_or(self.0, rhs.0), v128_or(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vorrq_u32(
                            core::mem::transmute(self.0),
                            core::mem::transmute(rhs.0),
                        )),
                        core::mem::transmute(vorrq_u32(
                            core::mem::transmute(self.1),
                            core::mem::transmute(rhs.1),
                        )),
                    )
                }
            } else {
                Self([
                    f32::from_bits(self.0[0].to_bits() | rhs.0[0].to_bits()),
                    f32::from_bits(self.0[1].to_bits() | rhs.0[1].to_bits()),
                    f32::from_bits(self.0[2].to_bits() | rhs.0[2].to_bits()),
                    f32::from_bits(self.0[3].to_bits() | rhs.0[3].to_bits()),
                    f32::from_bits(self.0[4].to_bits() | rhs.0[4].to_bits()),
                    f32::from_bits(self.0[5].to_bits() | rhs.0[5].to_bits()),
                    f32::from_bits(self.0[6].to_bits() | rhs.0[6].to_bits()),
                    f32::from_bits(self.0[7].to_bits() | rhs.0[7].to_bits()),
                ])
            }
        }
    }
}

impl core::ops::BitXor for f32x8 {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(unsafe { _mm256_xor_ps(self.0, rhs.0) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(
                    unsafe { _mm_xor_ps(self.0, rhs.0) },
                    unsafe { _mm_xor_ps(self.1, rhs.1) },
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_xor(self.0, rhs.0), v128_xor(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(veorq_u32(
                            core::mem::transmute(self.0),
                            core::mem::transmute(rhs.0),
                        )),
                        core::mem::transmute(veorq_u32(
                            core::mem::transmute(self.1),
                            core::mem::transmute(rhs.1),
                        )),
                    )
                }
            } else {
                Self([
                    f32::from_bits(self.0[0].to_bits() ^ rhs.0[0].to_bits()),
                    f32::from_bits(self.0[1].to_bits() ^ rhs.0[1].to_bits()),
                    f32::from_bits(self.0[2].to_bits() ^ rhs.0[2].to_bits()),
                    f32::from_bits(self.0[3].to_bits() ^ rhs.0[3].to_bits()),
                    f32::from_bits(self.0[4].to_bits() ^ rhs.0[4].to_bits()),
                    f32::from_bits(self.0[5].to_bits() ^ rhs.0[5].to_bits()),
                    f32::from_bits(self.0[6].to_bits() ^ rhs.0[6].to_bits()),
                    f32::from_bits(self.0[7].to_bits() ^ rhs.0[7].to_bits()),
                ])
            }
        }
    }
}

impl core::ops::Neg for f32x8 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::default() - self
    }
}

impl core::ops::Not for f32x8 {
    type Output = Self;

    fn not(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                let all_bits = unsafe { _mm256_set1_ps(f32::from_bits(u32::MAX)) };
                Self(unsafe { _mm256_xor_ps(self.0, all_bits) })
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                unsafe {
                    let all_bits = _mm_set1_ps(f32::from_bits(u32::MAX));
                    Self(_mm_xor_ps(self.0, all_bits), _mm_xor_ps(self.1, all_bits))
                }
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_not(self.0), v128_not(self.1))
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    Self(
                        core::mem::transmute(vmvnq_u32(core::mem::transmute(self.0))),
                        core::mem::transmute(vmvnq_u32(core::mem::transmute(self.1))),
                    )
                }
            } else {
                self ^ Self::splat(cast(u32::MAX))
            }
        }
    }
}

impl core::cmp::PartialEq for f32x8 {
    fn eq(&self, rhs: &Self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                let mask = unsafe { _mm256_cmp_ps(self.0, rhs.0, _CMP_EQ_OQ) };
                unsafe { _mm256_movemask_ps(mask) == 0b1111_1111 }
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                unsafe { _mm_movemask_ps(_mm_cmpeq_ps(self.0, rhs.0)) == 0b1111 }
            } else if #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
                unsafe {
                    vminvq_u32(vandq_u32(
                        vceqq_f32(self.0, rhs.0),
                        vceqq_f32(self.1, rhs.1),
                    )) != 0
                }
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                u32x4_all_true(f32x4_eq(self.0, rhs.0)) &
                u32x4_all_true(f32x4_eq(self.1, rhs.1))
            } else {
                self.0 == rhs.0
            }
        }
    }
}
