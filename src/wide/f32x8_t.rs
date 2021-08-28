// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on https://github.com/Lokathor/wide (Zlib)

use bytemuck::cast;

use crate::wide::{u32x8, i32x8};

#[cfg(all(not(feature = "std"), feature = "libm"))]
use crate::scalar::FloatExt;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "simd", target_feature = "avx"))] {
        use safe_arch::*;

        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8(m256);
    } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
        use safe_arch::*;

        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8(m128, m128);
    } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
        use core::arch::wasm32::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8(v128, v128);

        impl Default for f32x8 {
            fn default() -> Self {
                Self::splat(0.0)
            }
        }

        impl PartialEq for f32x8 {
            fn eq(&self, other: &Self) -> bool {
                u32x4_all_true(f32x4_eq(self.0, other.0)) &
                u32x4_all_true(f32x4_eq(self.1, other.1))
            }
        }
    } else {
        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        #[repr(C, align(32))]
        pub struct f32x8([f32; 8]);
    }
}

unsafe impl bytemuck::Zeroable for f32x8 {}
unsafe impl bytemuck::Pod for f32x8 {}

impl f32x8 {
    pub fn splat(n: f32) -> Self {
        cast([n, n, n, n, n, n, n, n])
    }

    pub fn floor(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_floor(self.0), f32x4_floor(self.1))
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
                Self(cmp_op_mask_m256!(self.0, EqualOrdered, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(cmp_eq_mask_m128(self.0, rhs.0), cmp_eq_mask_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_eq(self.0, rhs.0), f32x4_eq(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, eq, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_ge(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(cmp_op_mask_m256!(self.0, GreaterEqualOrdered, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(cmp_ge_mask_m128(self.0, rhs.0), cmp_ge_mask_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_ge(self.0, rhs.0), f32x4_ge(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, ge, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_gt(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(cmp_op_mask_m256!(self.0, GreaterThanOrdered, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(cmp_gt_mask_m128(self.0, rhs.0), cmp_gt_mask_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_gt(self.0, rhs.0), f32x4_gt(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, gt, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_ne(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(cmp_op_mask_m256!(self.0, NotEqualOrdered, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(cmp_neq_mask_m128(self.0, rhs.0), cmp_neq_mask_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_ne(self.0, rhs.0), f32x4_ne(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, ne, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_le(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(cmp_op_mask_m256!(self.0, LessEqualOrdered, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(cmp_le_mask_m128(self.0, rhs.0), cmp_le_mask_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_le(self.0, rhs.0), f32x4_le(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, le, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn cmp_lt(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(cmp_op_mask_m256!(self.0, LessThanOrdered, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(cmp_lt_mask_m128(self.0, rhs.0), cmp_lt_mask_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_lt(self.0, rhs.0), f32x4_lt(self.1, rhs.1))
            } else {
                Self(impl_x8_cmp!(self, lt, rhs, f32::from_bits(u32::MAX), 0.0))
            }
        }
    }

    pub fn blend(self, t: Self, f: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(blend_varying_m256(f.0, t.0, self.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                Self(blend_varying_m128(f.0, t.0, self.0), blend_varying_m128(f.1, t.1, self.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_bitselect(t.0, f.0, self.0), v128_bitselect(t.1, f.1, self.1))
            } else {
                super::generic_bit_blend(self, t, f)
            }
        }
    }

    pub fn abs(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_abs(self.0), f32x4_abs(self.1))
            } else {
                let non_sign_bits = f32x8::splat(f32::from_bits(i32::MAX as u32));
                self & non_sign_bits
            }
        }
    }

    pub fn max(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(max_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(max_m128(self.0, rhs.0), max_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_max(self.0, rhs.0), f32x4_max(self.1, rhs.1))
            } else {
                Self(impl_x8_op!(self, max, rhs))
            }
        }
    }

    pub fn min(self, rhs: Self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(min_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(min_m128(self.0, rhs.0), min_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_min(self.0, rhs.0), f32x4_min(self.1, rhs.1))
            } else {
                Self(impl_x8_op!(self, min, rhs))
            }
        }
    }

    pub fn is_finite(self) -> Self {
        let shifted_exp_mask = u32x8::splat(0xFF000000);
        let u: u32x8 = cast(self);
        let shift_u = u << 1;
        let out = !(shift_u & shifted_exp_mask).cmp_eq(shifted_exp_mask);
        cast(out)
    }

    pub fn round(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(round_m256!(self.0, Nearest))
            } else if #[cfg(all(feature = "simd", target_feature = "sse4.1"))] {
                Self(round_m128!(self.0, Nearest), round_m128!(self.1, Nearest))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_nearest(self.0), f32x4_nearest(self.1))
            } else {
                let to_int = f32x8::splat(1.0 / f32::EPSILON);
                let u: u32x8 = cast(self);
                let e: i32x8 = cast((u >> 23) & u32x8::splat(0xff));
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
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                cast(convert_to_i32_m256i_from_m256(self.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                i32x8(
                    convert_to_i32_m128i_from_m128(self.0),
                    convert_to_i32_m128i_from_m128(self.1),
                )
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                let rounded = self.round();
                i32x8(i32x4_trunc_sat_f32x4(rounded.0), i32x4_trunc_sat_f32x4(rounded.1))
            } else {
                let rounded: [f32; 8] = cast(self.round());
                let rounded_ints: i32x8 = cast([
                    rounded[0] as i32,
                    rounded[1] as i32,
                    rounded[2] as i32,
                    rounded[3] as i32,
                    rounded[4] as i32,
                    rounded[5] as i32,
                    rounded[6] as i32,
                    rounded[7] as i32,
                ]);
                cast::<f32x8, i32x8>(self.is_finite()).blend(
                    rounded_ints,
                    i32x8::splat(i32::MIN)
                )
            }
        }
    }

    pub fn trunc_int(self) -> i32x8 {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                cast(convert_truncate_to_i32_m256i_from_m256(self.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                i32x8(truncate_m128_to_m128i(self.0), truncate_m128_to_m128i(self.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                cast(Self(
                    i32x4_trunc_sat_f32x4(self.0),
                    i32x4_trunc_sat_f32x4(self.1),
                ))
            } else {
                let n: [f32; 8] = cast(self);
                let ints: i32x8 = cast([
                    n[0].trunc() as i32,
                    n[1].trunc() as i32,
                    n[2].trunc() as i32,
                    n[3].trunc() as i32,
                    n[4].trunc() as i32,
                    n[5].trunc() as i32,
                    n[6].trunc() as i32,
                    n[7].trunc() as i32,
                ]);
                cast::<f32x8, i32x8>(self.is_finite()).blend(ints,i32x8::splat(i32::MIN))
            }
        }
    }

    pub fn recip(self) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(reciprocal_m256(self.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(reciprocal_m128(self.0), reciprocal_m128(self.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                let one = f32x4_splat(1.0);
                Self(
                    f32x4_div(one, self.0),
                    f32x4_div(one, self.1),
                )
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
                Self(reciprocal_sqrt_m256(self.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(reciprocal_sqrt_m128(self.0), reciprocal_sqrt_m128(self.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                let one = f32x4_splat(1.0);
                Self(
                    f32x4_div(one, f32x4_sqrt(self.0)),
                    f32x4_div(one, f32x4_sqrt(self.1)),
                )
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
                Self(sqrt_m256(self.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(sqrt_m128(self.0), sqrt_m128(self.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_sqrt(self.0), f32x4_sqrt(self.1))
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
                Self(add_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(add_m128(self.0, rhs.0), add_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_add(self.0, rhs.0), f32x4_add(self.1, rhs.1))
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
                Self(sub_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(sub_m128(self.0, rhs.0), sub_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_sub(self.0, rhs.0), f32x4_sub(self.1, rhs.1))
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
                Self(mul_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(mul_m128(self.0, rhs.0), mul_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_mul(self.0, rhs.0), f32x4_mul(self.1, rhs.1))
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
                Self(div_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(div_m128(self.0, rhs.0), div_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(f32x4_div(self.0, rhs.0), f32x4_div(self.1, rhs.1))
            } else {
                Self(impl_x8_op!(self, div, rhs))
            }
        }
    }
}

impl core::ops::BitAnd for f32x8 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(bitand_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(bitand_m128(self.0, rhs.0), bitand_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_and(self.0, rhs.0), v128_and(self.1, rhs.1))
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

    fn bitor(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(bitor_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(bitor_m128(self.0, rhs.0), bitor_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_or(self.0, rhs.0), v128_or(self.1, rhs.1))
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

    fn bitxor(self, rhs: Self) -> Self::Output {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "simd", target_feature = "avx"))] {
                Self(bitxor_m256(self.0, rhs.0))
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(bitxor_m128(self.0, rhs.0), bitxor_m128(self.1, rhs.1))
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_xor(self.0, rhs.0), v128_xor(self.1, rhs.1))
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
                Self(self.0.not())
            } else if #[cfg(all(feature = "simd", target_feature = "sse2"))] {
                Self(self.0.not(), self.1.not())
            } else if #[cfg(all(feature = "simd", target_feature = "simd128"))] {
                Self(v128_not(self.0), v128_not(self.1))
            } else {
                self ^ Self::splat(cast(u32::MAX))
            }
        }
    }
}
