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
