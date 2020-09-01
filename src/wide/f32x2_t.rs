// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Right now, there are no visible benefits of using SIMD for F32x2. So we don't.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct F32x2([f32; 2]);

impl F32x2 {
    #[inline(always)]
    pub fn new(a: f32, b: f32) -> F32x2 {
        F32x2([a, b])
    }

    #[inline(always)]
    pub fn splat(x: f32) -> F32x2 {
        F32x2([x, x])
    }

    #[inline(always)] pub fn x(&self) -> f32 { self.0[0] }
    #[inline(always)] pub fn y(&self) -> f32 { self.0[1] }
}

impl std::ops::Add<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn add(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() + other.x(),
            self.y() + other.y(),
        ])
    }
}

impl std::ops::Sub<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn sub(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() - other.x(),
            self.y() - other.y(),
        ])
    }
}

impl std::ops::Mul<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn mul(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() * other.x(),
            self.y() * other.y(),
        ])
    }
}

impl std::ops::Div<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn div(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() / other.x(),
            self.y() / other.y(),
        ])
    }
}
