// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Right now, there are no visible benefits of using SIMD for f32x2. So we don't.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct f32x2([f32; 2]);

impl f32x2 {
    pub fn new(a: f32, b: f32) -> f32x2 {
        f32x2([a, b])
    }

    pub fn splat(x: f32) -> f32x2 {
        f32x2([x, x])
    }

    pub fn abs(self) -> f32x2 {
        f32x2([
            self.x().abs(),
            self.y().abs(),
        ])
    }

    pub fn min(self, other: f32x2) -> f32x2 {
        f32x2([
            self.x().min(other.x()),
            self.y().min(other.y()),
        ])
    }

    pub fn max(self, other: f32x2) -> f32x2 {
        f32x2([
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

impl std::ops::Add<f32x2> for f32x2 {
    type Output = f32x2;

    fn add(self, other: f32x2) -> f32x2 {
        f32x2([
            self.x() + other.x(),
            self.y() + other.y(),
        ])
    }
}

impl std::ops::Sub<f32x2> for f32x2 {
    type Output = f32x2;

    fn sub(self, other: f32x2) -> f32x2 {
        f32x2([
            self.x() - other.x(),
            self.y() - other.y(),
        ])
    }
}

impl std::ops::Mul<f32x2> for f32x2 {
    type Output = f32x2;

    fn mul(self, other: f32x2) -> f32x2 {
        f32x2([
            self.x() * other.x(),
            self.y() * other.y(),
        ])
    }
}

impl std::ops::Div<f32x2> for f32x2 {
    type Output = f32x2;

    fn div(self, other: f32x2) -> f32x2 {
        f32x2([
            self.x() / other.x(),
            self.y() / other.y(),
        ])
    }
}
