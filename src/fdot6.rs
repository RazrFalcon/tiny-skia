// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::convert::TryFrom;

use crate::fixed::{self, Fixed};
use crate::math::left_shift;

pub type FDot6 = i32;

pub const ONE: FDot6 = 64;

pub fn from_i32(n: i32) -> FDot6 {
    debug_assert!(n as i16 as i32 == n);
    n << 6
}

pub fn from_f32(n: f32) -> FDot6 {
    (n * 64.0) as i32
}

pub fn floor(n: FDot6) -> FDot6 {
    n >> 6
}

pub fn ceil(n: FDot6) -> FDot6 {
    (n + 63) >> 6
}

pub fn round(n: FDot6) -> FDot6 {
    (n + 32) >> 6
}

pub fn to_fixed(n: FDot6) -> Fixed {
    debug_assert!((left_shift(n, 10) >> 10) == n);
    left_shift(n, 10)
}

pub fn div(a: FDot6, b: FDot6) -> Fixed {
    debug_assert_ne!(b, 0);

    if i16::try_from(a).is_ok() {
        left_shift(a, 16) / b
    } else {
        fixed::div(a, b)
    }
}

pub fn can_convert_to_fixed(n: FDot6) -> bool {
    let max_dot6 = std::i32::MAX >> (16 - 6);
    n.abs() <= max_dot6
}

pub fn small_scale(value: u8, dot6: FDot6) -> u8 {
    debug_assert!(dot6 as u32 <= 64);
    ((value as i32 * dot6) >> 6) as u8
}
