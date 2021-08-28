// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This module was written from scratch, therefore there is no Google copyright.

// f32x16, i32x16 and u32x16 are implemented as [Tx8; 2] and not as [T; 16].
// This way we still can use some SIMD.
//
// We doesn't use #[inline] that much in this module.
// The compiler will inline most of the methods automatically.
// The only exception is U16x16, were we have to force inlining,
// otherwise the performance will be horrible.

#![allow(non_camel_case_types)]

#[allow(unused_macros)]
macro_rules! impl_x8_op {
    ($a:expr, $op:ident, $b:expr) => {[
        $a.0[0].$op($b.0[0]),
        $a.0[1].$op($b.0[1]),
        $a.0[2].$op($b.0[2]),
        $a.0[3].$op($b.0[3]),
        $a.0[4].$op($b.0[4]),
        $a.0[5].$op($b.0[5]),
        $a.0[6].$op($b.0[6]),
        $a.0[7].$op($b.0[7]),
    ]};
}

#[allow(unused_macros)]
macro_rules! impl_x8_cmp {
    ($a:expr, $op:ident, $b:expr, $passed:expr, $failed:expr) => {[
        if $a.0[0].$op(&$b.0[0]) { $passed } else { $failed },
        if $a.0[1].$op(&$b.0[1]) { $passed } else { $failed },
        if $a.0[2].$op(&$b.0[2]) { $passed } else { $failed },
        if $a.0[3].$op(&$b.0[3]) { $passed } else { $failed },
        if $a.0[4].$op(&$b.0[4]) { $passed } else { $failed },
        if $a.0[5].$op(&$b.0[5]) { $passed } else { $failed },
        if $a.0[6].$op(&$b.0[6]) { $passed } else { $failed },
        if $a.0[7].$op(&$b.0[7]) { $passed } else { $failed },
    ]};
}


mod f32x2_t;
mod f32x4_t;
mod f32x8_t;
mod i32x8_t;
mod u32x8_t;
mod f32x16_t;
mod u16x16_t;

pub use f32x2_t::f32x2;
pub use f32x4_t::f32x4;
pub use f32x8_t::f32x8;
pub use i32x8_t::i32x8;
pub use u32x8_t::u32x8;
pub use f32x16_t::f32x16;
pub use u16x16_t::u16x16;

#[allow(dead_code)]
pub fn generic_bit_blend<T>(mask: T, y: T, n: T) -> T
where
    T: Copy + core::ops::BitXor<Output = T> + core::ops::BitAnd<Output = T>,
{
    n ^ ((n ^ y) & mask)
}
