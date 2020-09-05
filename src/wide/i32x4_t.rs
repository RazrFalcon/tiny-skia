// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[cfg(all(feature = "sse2", target_arch = "x86"))]
use std::arch::x86;
#[cfg(all(feature = "sse2", target_arch = "x86_64"))]
use std::arch::x86_64 as x86;
#[cfg(all(feature = "sse2", target_feature = "sse2"))]
use x86::__m128i;

use super::{F32x4, U32x4};

#[cfg(all(feature = "sse2", target_feature = "sse2"))]
type Storage = __m128i;

#[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
type Storage = [i32; 4];

#[derive(Copy, Clone)]
pub struct I32x4(pub Storage);

impl I32x4 {
    #[cfg(feature = "sse2")]
    pub fn splat(n: i32) -> Self {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            I32x4(x86::_mm_set1_epi32(n))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            I32x4([n, n, n, n])
        }
    }

    pub fn as_slice(&self) -> &[i32; 4] {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            &*(&self.0 as *const __m128i as *const [i32; 4])
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            &self.0
        }
    }

    pub fn to_f32x4(&self) -> F32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        unsafe {
            F32x4(x86::_mm_cvtepi32_ps(self.0))
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        {
            F32x4([
                self.x() as f32,
                self.y() as f32,
                self.z() as f32,
                self.w() as f32,
            ])
        }
    }

    pub fn to_u32x4(&self) -> U32x4 {
        #[cfg(all(feature = "sse2", target_feature = "sse2"))]
        {
            U32x4(self.0)
        }

        #[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
        unsafe {
            U32x4(std::mem::transmute::<[i32; 4], [u32; 4]>(self.0))
        }
    }

    pub fn x(&self) -> i32 { self.as_slice()[0] }
    pub fn y(&self) -> i32 { self.as_slice()[1] }
    pub fn z(&self) -> i32 { self.as_slice()[2] }
    pub fn w(&self) -> i32 { self.as_slice()[3] }
}

impl std::fmt::Debug for I32x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "I32x4({:?})", self.as_slice())
    }
}
