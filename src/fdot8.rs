// Copyright 2011 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Extracted from SkScan_Antihair.cpp

use crate::fixed::Fixed;

/// 24.8 integer fixed point
pub type FDot8 = i32;

pub fn from_fixed(x: Fixed) -> FDot8 {
    (x + 0x80) >> 8
}
