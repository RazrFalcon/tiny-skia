// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[inline]
pub(crate) fn left_shift(value: i32, shift: i32) -> i32 {
    ((value as u32) << shift) as i32
}

#[inline]
pub(crate) fn left_shift64(value: i64, shift: i32) -> i64 {
    ((value as u64) << shift) as i64
}

#[inline]
pub(crate) fn bound<T: Ord + Copy>(min: T, value: T, max: T) -> T {
    max.min(value).max(min)
}
