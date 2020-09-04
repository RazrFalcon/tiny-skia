// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[cfg(all(feature = "sse2", target_arch = "x86"))]
use std::arch::x86;
#[cfg(all(feature = "sse2", target_arch = "x86_64"))]
use std::arch::x86_64 as x86;
#[cfg(all(feature = "sse2", target_feature = "sse2"))]
use x86::{__m128, __m128i};

use super::F32x4;

#[cfg(all(feature = "sse2", target_feature = "sse2"))]
type Storage = __m128i;

#[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
type Storage = [u32; 4];

#[derive(Copy, Clone)]
pub struct U32x4(pub Storage);

impl U32x4 {
    pub fn as_slice(&self) -> &[u32; 4] {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            &*(&self.0 as *const __m128i as *const [u32; 4])
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            &self.0
        }
    }

    /// Returns true if all four booleans in this vector are true.
    ///
    /// The result is *undefined* if all four values in this vector are not booleans. A boolean is
    /// a value with all bits set or all bits clear (i.e. !0 or 0).
    #[cfg(all(feature = "sse2", target_feature = "sse2"))]
    pub fn all_true(self) -> bool {
        unsafe { x86::_mm_movemask_ps(x86::_mm_castsi128_ps(self.0)) == 0x0f }
    }

    pub fn if_then_else(&self, t: F32x4, e: F32x4) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            // Do not use _mm_cvtepi32_ps here. We need `self` as bits, no as float.
            let c = *(&self.0 as *const __m128i as *const __m128);
            F32x4(x86::_mm_or_ps(x86::_mm_and_ps(c, t.0), x86::_mm_andnot_ps(c, e.0)))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                if self.0[0] != 0 { t.0[0] } else { e.0[0] },
                if self.0[1] != 0 { t.0[1] } else { e.0[1] },
                if self.0[2] != 0 { t.0[2] } else { e.0[2] },
                if self.0[3] != 0 { t.0[3] } else { e.0[3] },
            ])
        }
    }
}

impl std::fmt::Debug for U32x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "U32x4({:?})", self.as_slice())
    }
}
