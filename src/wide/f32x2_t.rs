// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Right now, there are no visible benefits of using SIMD for F32x2. So we don't.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct F32x2([f32; 2]);

impl F32x2 {
    pub fn new(a: f32, b: f32) -> F32x2 {
        F32x2([a, b])
    }

    pub fn splat(x: f32) -> F32x2 {
        F32x2([x, x])
    }

    pub fn abs(self) -> F32x2 {
        F32x2([
            self.x().abs(),
            self.y().abs(),
        ])
    }

    pub fn min(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x().min(other.x()),
            self.y().min(other.y()),
        ])
    }

    pub fn max(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x().max(other.x()),
            self.y().max(other.y()),
        ])
    }

    pub fn max_component(self) -> f32 {
        self.x().max(self.y())
    }

    pub fn x(&self) -> f32 { self.0[0] }
    pub fn y(&self) -> f32 { self.0[1] }
}

impl std::ops::Add<F32x2> for F32x2 {
    type Output = F32x2;

    fn add(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() + other.x(),
            self.y() + other.y(),
        ])
    }
}

impl std::ops::Sub<F32x2> for F32x2 {
    type Output = F32x2;

    fn sub(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() - other.x(),
            self.y() - other.y(),
        ])
    }
}

impl std::ops::Mul<F32x2> for F32x2 {
    type Output = F32x2;

    fn mul(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() * other.x(),
            self.y() * other.y(),
        ])
    }
}

impl std::ops::Div<F32x2> for F32x2 {
    type Output = F32x2;

    fn div(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() / other.x(),
            self.y() / other.y(),
        ])
    }
}
