// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This module was written from scratch, therefore there is no Google copyright.

// f32x16, i32x16 and u32x16 are implemented as [Tx4; 4] and not as [T; 16].
// This way we still can use some SSE2.
//
// We doesn't use #[inline] that much in this module.
// The compiler will inline most of the methods automatically.
// The only exception is U16x16, were we have to force inlining,
// otherwise the performance will be horrible.

pub use wide::{f32x4, i32x4, u32x4, f32x8, i32x8, u32x8};

mod u16x16_t;
mod f32x2_t;
mod f32x16_t;
mod i32x16_t;
mod u32x16_t;

pub use u16x16_t::u16x16;
pub use f32x2_t::f32x2;
pub use f32x16_t::f32x16;
pub use i32x16_t::i32x16;
pub use u32x16_t::u32x16;


pub trait F32x4Ext {
    fn floor(self) -> Self;
    fn fract(self) -> Self;
    fn normalize(self) -> Self;
    fn to_i32x4_bitcast(self) -> i32x4;
    fn to_u32x4_bitcast(self) -> u32x4;
}

impl F32x4Ext for f32x4 {
    fn floor(self) -> Self {
        use wide::CmpGt;
        let roundtrip = self.trunc_int().round_float();
        roundtrip - roundtrip.cmp_gt(self).blend(f32x4::splat(1.0), f32x4::default())
    }

    fn fract(self) -> Self {
        self - self.floor()
    }

    fn normalize(self) -> Self {
        self.max(f32x4::default()).min(f32x4::splat(1.0))
    }

    fn to_i32x4_bitcast(self) -> i32x4 {
        bytemuck::cast(self)
    }

    fn to_u32x4_bitcast(self) -> u32x4 {
        bytemuck::cast(self)
    }
}

pub trait U32x4Ext {
    fn to_i32x4_bitcast(self) -> i32x4;
    fn to_f32x4_bitcast(self) -> f32x4;
}

impl U32x4Ext for u32x4 {
    fn to_i32x4_bitcast(self) -> i32x4 {
        bytemuck::cast(self)
    }

    fn to_f32x4_bitcast(self) -> f32x4 {
        bytemuck::cast(self)
    }
}

pub trait I32x4Ext {
    fn to_u32x4_bitcast(self) -> u32x4;
    fn to_f32x4_bitcast(self) -> f32x4;
}

impl I32x4Ext for i32x4 {
    fn to_u32x4_bitcast(self) -> u32x4 {
        bytemuck::cast(self)
    }

    fn to_f32x4_bitcast(self) -> f32x4 {
        bytemuck::cast(self)
    }
}


pub trait F32x8Ext {
    fn floor(self) -> Self;
    fn fract(self) -> Self;
    fn normalize(self) -> Self;
    fn to_i32x8_bitcast(self) -> i32x8;
    fn to_u32x8_bitcast(self) -> u32x8;
}

impl F32x8Ext for f32x8 {
    fn floor(self) -> Self {
        use wide::CmpGt;
        let roundtrip = self.trunc_int().round_float();
        roundtrip - roundtrip.cmp_gt(self).blend(f32x8::splat(1.0), f32x8::default())
    }

    fn fract(self) -> Self {
        self - self.floor()
    }

    fn normalize(self) -> Self {
        self.max(f32x8::default()).min(f32x8::splat(1.0))
    }

    fn to_i32x8_bitcast(self) -> i32x8 {
        bytemuck::cast(self)
    }

    fn to_u32x8_bitcast(self) -> u32x8 {
        bytemuck::cast(self)
    }
}

pub trait U32x8Ext {
    fn to_i32x8_bitcast(self) -> i32x8;
    fn to_f32x8_bitcast(self) -> f32x8;
}

impl U32x8Ext for u32x8 {
    fn to_i32x8_bitcast(self) -> i32x8 {
        bytemuck::cast(self)
    }

    fn to_f32x8_bitcast(self) -> f32x8 {
        bytemuck::cast(self)
    }
}

pub trait I32x8Ext {
    fn to_u32x8_bitcast(self) -> u32x8;
    fn to_f32x8_bitcast(self) -> f32x8;
}

impl I32x8Ext for i32x8 {
    fn to_u32x8_bitcast(self) -> u32x8 {
        bytemuck::cast(self)
    }

    fn to_f32x8_bitcast(self) -> f32x8 {
        bytemuck::cast(self)
    }
}
