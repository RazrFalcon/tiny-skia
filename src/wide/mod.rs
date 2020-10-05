// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This module was written from scratch, therefore there is no Google copyright.

// Some ideas were taken from `pathfinder_simd` (Apache 2.0/MIT).

// F32x16, I32x16 and U32x16 are implemented as [Tx4; 4] and not as [T; 16].
// This way we still can use some SSE2.
//
// We doesn't use #[inline] that much in this module.
// The compiler will inline most of the methods automatically.
// The only exception is U16x16, were we have to force inlining,
// otherwise the performance will be horrible.

mod u16x16_t;

mod f32x2_t;
mod f32x4_t;
mod f32x16_t;

mod i32x4_t;
mod i32x16_t;

mod u32x4_t;
mod u32x16_t;

pub use u16x16_t::U16x16;

pub use f32x2_t::F32x2;
pub use f32x4_t::F32x4;
pub use f32x16_t::F32x16;

pub use i32x4_t::I32x4;
pub use i32x16_t::I32x16;

pub use u32x4_t::U32x4;
pub use u32x16_t::U32x16;
