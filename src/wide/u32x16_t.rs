// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{U32x4, F32x16};

#[derive(Copy, Clone)]
pub struct U32x16(pub [U32x4; 4]);

impl U32x16 {
    pub fn if_then_else(&self, t: F32x16, e: F32x16) -> F32x16 {
        F32x16([
            self.0[0].if_then_else(t.0[0], e.0[0]),
            self.0[1].if_then_else(t.0[1], e.0[1]),
            self.0[2].if_then_else(t.0[2], e.0[2]),
            self.0[3].if_then_else(t.0[3], e.0[3]),
        ])
    }
}

impl std::fmt::Debug for U32x16 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "U32x16({} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {})",
               self.0[0].x(), self.0[0].y(), self.0[0].z(), self.0[0].w(),
               self.0[1].x(), self.0[1].y(), self.0[1].z(), self.0[1].w(),
               self.0[2].x(), self.0[2].y(), self.0[2].z(), self.0[2].w(),
               self.0[3].x(), self.0[3].y(), self.0[3].z(), self.0[3].w(),
        )
    }
}
