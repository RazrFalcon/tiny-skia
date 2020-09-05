// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{I32x4, F32x16};

#[derive(Copy, Clone)]
pub struct I32x16(pub [I32x4; 4]);

impl I32x16 {
    pub fn to_f32x16(&self) -> F32x16 {
        F32x16([
            self.0[0].to_f32x4(),
            self.0[1].to_f32x4(),
            self.0[2].to_f32x4(),
            self.0[3].to_f32x4(),
        ])
    }
}

impl std::fmt::Debug for I32x16 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "I32x16({} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {})",
               self.0[0].x(), self.0[0].y(), self.0[0].z(), self.0[0].w(),
               self.0[1].x(), self.0[1].y(), self.0[1].z(), self.0[1].w(),
               self.0[2].x(), self.0[2].y(), self.0[2].z(), self.0[2].w(),
               self.0[3].x(), self.0[3].y(), self.0[3].z(), self.0[3].w(),
        )
    }
}
