// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Loosely based on `pathfinder_simd` (Apache 2.0/MIT).

mod f32x2_t;
mod f32x4_t;
mod i32x4_t;
mod u16x16_t;
mod u32x4_t;

pub use f32x2_t::F32x2;
pub use f32x4_t::F32x4;
pub use i32x4_t::I32x4;
pub use u16x16_t::U16x16;
pub use u32x4_t::U32x4;
