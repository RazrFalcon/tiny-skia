// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This a bare minimum `pathfinder_simd` fork (Apache 2.0/MIT)
// that acts more like Skia's Sk4s type.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    #[cfg(target_pointer_width = "32")]
    use std::arch::x86::{__m128, __m128i};
    #[cfg(target_pointer_width = "32")]
    use std::arch::x86;
    #[cfg(target_pointer_width = "64")]
    use std::arch::x86_64::{__m128, __m128i};
    #[cfg(target_pointer_width = "64")]
    use std::arch::x86_64 as x86;

    #[derive(Copy, Clone)]
    struct U32x4(__m128i);

    impl U32x4 {
        /// Returns true if all four booleans in this vector are true.
        ///
        /// The result is *undefined* if all four values in this vector are not booleans. A boolean is
        /// a value with all bits set or all bits clear (i.e. !0 or 0).
        #[inline(always)]
        fn all_true(self) -> bool {
            unsafe { x86::_mm_movemask_ps(x86::_mm_castsi128_ps(self.0)) == 0x0f }
        }
    }

    #[derive(Copy, Clone)]
    pub struct F32x4(__m128);

    impl F32x4 {
        #[inline(always)]
        pub fn new(a: f32, b: f32, c: f32, d: f32) -> F32x4 {
            unsafe {
                let vector = [a, b, c, d];
                F32x4(x86::_mm_loadu_ps(vector.as_ptr()))
            }
        }

        // #[inline(always)]
        // pub fn splat(x: f32) -> F32x4 {
        //     unsafe { F32x4(x86::_mm_set1_ps(x)) }
        // }

        #[inline(always)]
        pub fn min(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_min_ps(self.0, other.0)) }
        }

        #[inline(always)]
        pub fn max(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_max_ps(self.0, other.0)) }
        }

        #[inline(always)]
        fn packed_eq(self, other: F32x4) -> U32x4 {
            unsafe {
                U32x4(x86::_mm_castps_si128(x86::_mm_cmpeq_ps(
                    self.0, other.0,
                )))
            }
        }

        #[inline(always)]
        pub fn as_slice(&self) -> &[f32; 4] {
            unsafe { &*(&self.0 as *const __m128 as *const [f32; 4]) }
        }

        #[inline(always)] pub fn x(&self) -> f32 { self.as_slice()[0] }
        #[inline(always)] pub fn y(&self) -> f32 { self.as_slice()[1] }
        #[inline(always)] pub fn z(&self) -> f32 { self.as_slice()[2] }
        #[inline(always)] pub fn w(&self) -> f32 { self.as_slice()[3] }
    }

    impl Default for F32x4 {
        #[inline(always)]
        fn default() -> F32x4 {
            unsafe { F32x4(x86::_mm_setzero_ps()) }
        }
    }

    impl std::fmt::Debug for F32x4 {
        #[inline(always)]
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            write!(f, "F32x4({:?})", self.as_slice())
        }
    }

    impl PartialEq for F32x4 {
        #[inline(always)]
        fn eq(&self, other: &F32x4) -> bool {
            self.packed_eq(*other).all_true()
        }
    }

    impl std::ops::Add<F32x4> for F32x4 {
        type Output = F32x4;
        #[inline(always)]
        fn add(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_add_ps(self.0, other.0)) }
        }
    }

    impl std::ops::Div<F32x4> for F32x4 {
        type Output = F32x4;
        #[inline(always)]
        fn div(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_div_ps(self.0, other.0)) }
        }
    }

    impl std::ops::Mul<F32x4> for F32x4 {
        type Output = F32x4;
        #[inline(always)]
        fn mul(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_mul_ps(self.0, other.0)) }
        }
    }

    impl std::ops::MulAssign for F32x4 {
        #[inline(always)]
        fn mul_assign(&mut self, other: F32x4) {
            *self = *self * other
        }
    }

    impl std::ops::Sub<F32x4> for F32x4 {
        type Output = F32x4;
        #[inline(always)]
        fn sub(self, other: F32x4) -> F32x4 {
            unsafe { F32x4(x86::_mm_sub_ps(self.0, other.0)) }
        }
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod scalar {
    #[derive(Copy, Clone, Default, PartialEq, Debug)]
    pub struct F32x4([f32; 4]);

    impl F32x4 {
        #[inline(always)]
        pub fn new(a: f32, b: f32, c: f32, d: f32) -> F32x4 {
            F32x4([a, b, c, d])
        }

        // #[inline(always)]
        // pub fn splat(x: f32) -> F32x4 {
        //     F32x4([x; 4])
        // }

        #[inline(always)]
        pub fn min(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x().min(other.x()),
                self.y().min(other.y()),
                self.z().min(other.z()),
                self.w().min(other.w()),
            ])
        }

        #[inline(always)]
        pub fn max(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x().max(other.x()),
                self.y().max(other.y()),
                self.z().max(other.z()),
                self.w().max(other.w()),
            ])
        }

        #[inline(always)]
        pub fn as_slice(&self) -> &[f32; 4] { &self.0 }

        #[inline(always)] pub fn x(&self) -> f32 { self.0[0] }
        #[inline(always)] pub fn y(&self) -> f32 { self.0[1] }
        #[inline(always)] pub fn z(&self) -> f32 { self.0[2] }
        #[inline(always)] pub fn w(&self) -> f32 { self.0[3] }
    }

    impl std::ops::Add<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn add(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() + other.x(),
                self.y() + other.y(),
                self.z() + other.z(),
                self.w() + other.w(),
            ])
        }
    }

    impl std::ops::Sub<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn sub(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() - other.x(),
                self.y() - other.y(),
                self.z() - other.z(),
                self.w() - other.w(),
            ])
        }
    }

    impl std::ops::Mul<F32x4> for F32x4 {
        type Output = F32x4;
        #[inline(always)]
        fn mul(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() * other.x(),
                self.y() * other.y(),
                self.z() * other.z(),
                self.w() * other.w(),
            ])
        }
    }

    impl std::ops::MulAssign for F32x4 {
        #[inline(always)]
        fn mul_assign(&mut self, other: F32x4) {
            *self = *self * other
        }
    }

    impl std::ops::Div<F32x4> for F32x4 {
        type Output = F32x4;

        #[inline(always)]
        fn div(self, other: F32x4) -> F32x4 {
            F32x4([
                self.x() / other.x(),
                self.y() / other.y(),
                self.z() / other.z(),
                self.w() / other.w(),
            ])
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86::F32x4;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use scalar::F32x4;


// Right now, there are no visible benefits of using SIMD for F32x2. So we don't.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct F32x2([f32; 2]);

impl F32x2 {
    #[inline(always)]
    pub fn new(a: f32, b: f32) -> F32x2 {
        F32x2([a, b])
    }

    #[inline(always)]
    pub fn splat(x: f32) -> F32x2 {
        F32x2([x, x])
    }

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.0[1]
    }
}

impl std::ops::Add<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn add(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() + other.x(),
            self.y() + other.y(),
        ])
    }
}

impl std::ops::Sub<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn sub(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() - other.x(),
            self.y() - other.y(),
        ])
    }
}

impl std::ops::Mul<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn mul(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() * other.x(),
            self.y() * other.y(),
        ])
    }
}

impl std::ops::Div<F32x2> for F32x2 {
    type Output = F32x2;

    #[inline(always)]
    fn div(self, other: F32x2) -> F32x2 {
        F32x2([
            self.x() / other.x(),
            self.y() / other.y(),
        ])
    }
}
