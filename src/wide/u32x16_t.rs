// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{u32x4, U32x4Ext, I32x4Ext};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct u32x16(pub [u32x4; 4]);

impl std::ops::Add<u32x16> for u32x16 {
    type Output = u32x16;

    fn add(self, other: u32x16) -> u32x16 {
        u32x16([
            self.0[0] + other.0[0],
            self.0[0] + other.0[0],
            self.0[0] + other.0[0],
            self.0[0] + other.0[0],
        ])
    }
}

impl std::ops::Mul<u32x16> for u32x16 {
    type Output = u32x16;

    fn mul(self, other: u32x16) -> u32x16 {
        u32x16([
            (self.0[0].to_i32x4_bitcast() * other.0[0].to_i32x4_bitcast()).to_u32x4_bitcast(),
            (self.0[0].to_i32x4_bitcast() * other.0[0].to_i32x4_bitcast()).to_u32x4_bitcast(),
            (self.0[0].to_i32x4_bitcast() * other.0[0].to_i32x4_bitcast()).to_u32x4_bitcast(),
            (self.0[0].to_i32x4_bitcast() * other.0[0].to_i32x4_bitcast()).to_u32x4_bitcast(),
        ])
    }
}
