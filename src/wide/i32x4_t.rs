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

#[cfg(all(feature = "sse2", target_feature = "sse2"))]
type Storage = __m128i;

#[cfg(not(all(feature = "sse2", target_feature = "sse2")))]
type Storage = [i32; 4];

#[derive(Copy, Clone)]
pub struct I32x4(pub Storage);

impl I32x4 {
    #[inline(always)]
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

    #[inline(always)] pub fn x(&self) -> i32 { self.as_slice()[0] }
    #[inline(always)] pub fn y(&self) -> i32 { self.as_slice()[1] }
    #[inline(always)] pub fn z(&self) -> i32 { self.as_slice()[2] }
    #[inline(always)] pub fn w(&self) -> i32 { self.as_slice()[3] }
}

impl std::fmt::Debug for I32x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "I32x4({:?})", self.as_slice())
    }
}
