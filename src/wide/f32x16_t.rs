// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{I32x16, U32x16, F32x4};
use crate::wide::U16x16;

#[derive(Copy, Clone, Default, PartialEq)]
pub struct F32x16(pub [F32x4; 4]);

impl F32x16 {
    pub fn splat(n: f32) -> Self {
        F32x16([F32x4::splat(n), F32x4::splat(n), F32x4::splat(n), F32x4::splat(n)])
    }

    #[inline]
    pub fn abs(&self) -> Self {
        // Yes, Skia does it in the same way.
        let abs = |x| bytemuck::cast::<i32, f32>(bytemuck::cast::<f32, i32>(x) & 0x7fffffff);

        F32x16([
            F32x4::new(
                abs(self.0[0].as_slice()[0]),
                abs(self.0[0].as_slice()[1]),
                abs(self.0[0].as_slice()[2]),
                abs(self.0[0].as_slice()[3]),
            ),
            F32x4::new(
                abs(self.0[1].as_slice()[0]),
                abs(self.0[1].as_slice()[1]),
                abs(self.0[1].as_slice()[2]),
                abs(self.0[1].as_slice()[3]),
            ),
            F32x4::new(
                abs(self.0[2].as_slice()[0]),
                abs(self.0[2].as_slice()[1]),
                abs(self.0[2].as_slice()[2]),
                abs(self.0[2].as_slice()[3]),
            ),
            F32x4::new(
                abs(self.0[3].as_slice()[0]),
                abs(self.0[3].as_slice()[1]),
                abs(self.0[3].as_slice()[2]),
                abs(self.0[3].as_slice()[3]),
            ),
        ])
    }

    pub fn packed_gt(self, other: &Self) -> U32x16 {
        U32x16([
            self.0[0].packed_gt(other.0[0]),
            self.0[1].packed_gt(other.0[1]),
            self.0[2].packed_gt(other.0[2]),
            self.0[3].packed_gt(other.0[3]),
        ])
    }

    pub fn normalize(&self) -> Self {
        F32x16([
            self.0[0].normalize(),
            self.0[1].normalize(),
            self.0[2].normalize(),
            self.0[3].normalize(),
        ])
    }

    pub fn floor(&self) -> Self {
        // Yes, Skia does it in the same way.
        let roundtrip = self.to_i32x16_round().to_f32x16();
        roundtrip - (roundtrip.packed_gt(self))
            .if_then_else(F32x16::splat(1.0), F32x16::splat(0.0))
    }

    pub fn sqrt(&self) -> Self {
        F32x16([
            self.0[0].sqrt(),
            self.0[1].sqrt(),
            self.0[2].sqrt(),
            self.0[3].sqrt(),
        ])
    }

    pub fn to_i32x16_round(&self) -> I32x16 {
        I32x16([
            self.0[0].to_i32x4_round(),
            self.0[1].to_i32x4_round(),
            self.0[2].to_i32x4_round(),
            self.0[3].to_i32x4_round(),
        ])
    }

    // This method is too heavy. It shouldn't be inlined.
    pub fn save_to_u16x16(&self, dst: &mut U16x16) {
        // Do not use to_i32x4, because it involves rounding,
        // and Skia cast's without it.

        dst.0[ 0] = self.0[0].x() as u16;
        dst.0[ 1] = self.0[0].y() as u16;
        dst.0[ 2] = self.0[0].z() as u16;
        dst.0[ 3] = self.0[0].w() as u16;

        dst.0[ 4] = self.0[1].x() as u16;
        dst.0[ 5] = self.0[1].y() as u16;
        dst.0[ 6] = self.0[1].z() as u16;
        dst.0[ 7] = self.0[1].w() as u16;

        dst.0[ 8] = self.0[2].x() as u16;
        dst.0[ 9] = self.0[2].y() as u16;
        dst.0[10] = self.0[2].z() as u16;
        dst.0[11] = self.0[2].w() as u16;

        dst.0[12] = self.0[3].x() as u16;
        dst.0[13] = self.0[3].y() as u16;
        dst.0[14] = self.0[3].z() as u16;
        dst.0[15] = self.0[3].w() as u16;
    }
}

impl std::ops::Add<F32x16> for F32x16 {
    type Output = Self;

    fn add(self, other: F32x16) -> Self::Output {
        F32x16([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }
}

impl std::ops::Sub<F32x16> for F32x16 {
    type Output = Self;

    fn sub(self, other: F32x16) -> Self::Output {
        F32x16([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
        ])
    }
}

impl std::ops::Mul<F32x16> for F32x16 {
    type Output = Self;

    fn mul(self, other: F32x16) -> Self::Output {
        F32x16([
            self.0[0] * other.0[0],
            self.0[1] * other.0[1],
            self.0[2] * other.0[2],
            self.0[3] * other.0[3],
        ])
    }
}

impl std::fmt::Debug for F32x16 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "F32x16({} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {})",
            self.0[0].x(), self.0[0].y(), self.0[0].z(), self.0[0].w(),
            self.0[1].x(), self.0[1].y(), self.0[1].z(), self.0[1].w(),
            self.0[2].x(), self.0[2].y(), self.0[2].z(), self.0[2].w(),
            self.0[3].x(), self.0[3].y(), self.0[3].z(), self.0[3].w(),
        )
    }
}
