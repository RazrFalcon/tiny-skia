// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use wide::CmpGt;

use super::{f32x4, i32x16, u16x16, F32x4Ext};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct f32x16(pub [f32x4; 4]);

impl f32x16 {
    pub fn splat(n: f32) -> Self {
        f32x16([f32x4::splat(n), f32x4::splat(n), f32x4::splat(n), f32x4::splat(n)])
    }

    #[inline]
    pub fn abs(&self) -> Self {
        // Yes, Skia does it in the same way.
        let abs = |x| bytemuck::cast::<i32, f32>(bytemuck::cast::<f32, i32>(x) & 0x7fffffff);

        let n0: [f32; 4] = self.0[0].into();
        let n1: [f32; 4] = self.0[1].into();
        let n2: [f32; 4] = self.0[2].into();
        let n3: [f32; 4] = self.0[3].into();
        f32x16([
            f32x4::from([
                abs(n0[0]),
                abs(n0[1]),
                abs(n0[2]),
                abs(n0[3]),
            ]),
            f32x4::from([
                abs(n1[0]),
                abs(n1[1]),
                abs(n1[2]),
                abs(n1[3]),
            ]),
            f32x4::from([
                abs(n2[0]),
                abs(n2[1]),
                abs(n2[2]),
                abs(n2[3]),
            ]),
            f32x4::from([
                abs(n3[0]),
                abs(n3[1]),
                abs(n3[2]),
                abs(n3[3]),
            ]),
        ])
    }

    pub fn cmp_gt(self, other: &Self) -> Self {
        f32x16([
            self.0[0].cmp_gt(other.0[0]),
            self.0[1].cmp_gt(other.0[1]),
            self.0[2].cmp_gt(other.0[2]),
            self.0[3].cmp_gt(other.0[3]),
        ])
    }

    pub fn blend(self, t: Self, f: Self) -> Self {
        f32x16([
            self.0[0].blend(t.0[0], f.0[0]),
            self.0[1].blend(t.0[1], f.0[1]),
            self.0[2].blend(t.0[2], f.0[2]),
            self.0[3].blend(t.0[3], f.0[3]),
        ])
    }

    pub fn normalize(&self) -> Self {
        f32x16([
            self.0[0].normalize(),
            self.0[1].normalize(),
            self.0[2].normalize(),
            self.0[3].normalize(),
        ])
    }

    pub fn floor(&self) -> Self {
        // Yes, Skia does it in the same way.
        let roundtrip = self.to_i32x16_round().to_f32x16();
        roundtrip - roundtrip.cmp_gt(self).blend(f32x16::splat(1.0), f32x16::splat(0.0))
    }

    pub fn sqrt(&self) -> Self {
        f32x16([
            self.0[0].sqrt(),
            self.0[1].sqrt(),
            self.0[2].sqrt(),
            self.0[3].sqrt(),
        ])
    }

    pub fn to_i32x16_round(&self) -> i32x16 {
        i32x16([
            self.0[0].round_int(),
            self.0[1].round_int(),
            self.0[2].round_int(),
            self.0[3].round_int(),
        ])
    }

    // This method is too heavy. It shouldn't be inlined.
    pub fn save_to_u16x16(&self, dst: &mut u16x16) {
        // Do not use to_i32x4, because it involves rounding,
        // and Skia cast's without it.

        let n0: [f32; 4] = self.0[0].into();
        let n1: [f32; 4] = self.0[1].into();
        let n2: [f32; 4] = self.0[2].into();
        let n3: [f32; 4] = self.0[3].into();

        dst.0[ 0] = n0[0] as u16;
        dst.0[ 1] = n0[1] as u16;
        dst.0[ 2] = n0[2] as u16;
        dst.0[ 3] = n0[3] as u16;

        dst.0[ 4] = n1[0] as u16;
        dst.0[ 5] = n1[1] as u16;
        dst.0[ 6] = n1[2] as u16;
        dst.0[ 7] = n1[3] as u16;

        dst.0[ 8] = n2[0] as u16;
        dst.0[ 9] = n2[1] as u16;
        dst.0[10] = n2[2] as u16;
        dst.0[11] = n2[3] as u16;

        dst.0[12] = n3[0] as u16;
        dst.0[13] = n3[1] as u16;
        dst.0[14] = n3[2] as u16;
        dst.0[15] = n3[3] as u16;
    }
}

impl std::ops::Add<f32x16> for f32x16 {
    type Output = Self;

    fn add(self, other: f32x16) -> Self::Output {
        f32x16([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }
}

impl std::ops::Sub<f32x16> for f32x16 {
    type Output = Self;

    fn sub(self, other: f32x16) -> Self::Output {
        f32x16([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
        ])
    }
}

impl std::ops::Mul<f32x16> for f32x16 {
    type Output = Self;

    fn mul(self, other: f32x16) -> Self::Output {
        f32x16([
            self.0[0] * other.0[0],
            self.0[1] * other.0[1],
            self.0[2] * other.0[2],
            self.0[3] * other.0[3],
        ])
    }
}
