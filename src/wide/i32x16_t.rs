// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{i32x4, f32x16};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct i32x16(pub [i32x4; 4]);

impl i32x16 {
    pub fn to_f32x16(&self) -> f32x16 {
        f32x16([
            self.0[0].round_float(),
            self.0[1].round_float(),
            self.0[2].round_float(),
            self.0[3].round_float(),
        ])
    }
}
