// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::NormalizedF32;

/// 8-bit type for an alpha value. 255 is 100% opaque, zero is 100% transparent.
pub type AlphaU8 = u8;

/// Represents fully transparent AlphaU8 value.
pub const ALPHA_U8_TRANSPARENT: AlphaU8 = 0x00;

/// Represents fully opaque AlphaU8 value.
pub const ALPHA_U8_OPAQUE: AlphaU8 = 0xFF;

/// Represents fully transparent Alpha value.
pub const ALPHA_TRANSPARENT: NormalizedF32 = NormalizedF32::ZERO;

/// Represents fully opaque Alpha value.
pub const ALPHA_OPAQUE: NormalizedF32 = NormalizedF32::ONE;

/// A 32-bit RGBA color value.
///
/// Byteorder: ABGR
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct ColorU8(u32);

impl ColorU8 {
    /// Creates a new color.
    #[inline]
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        ColorU8(pack_rgba(r, g, b, a))
    }

    /// Returns color's red component.
    #[inline]
    pub const fn red(self) -> u8 {
        ((self.0 >> 0) & 0xFF) as u8
    }

    /// Returns color's green component.
    #[inline]
    pub const fn green(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// Returns color's blue component.
    #[inline]
    pub const fn blue(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// Returns color's alpha component.
    #[inline]
    pub const fn alpha(self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    /// Check that color is opaque.
    ///
    /// Alpha == 255
    #[inline]
    pub fn is_opaque(&self) -> bool {
        self.alpha() == ALPHA_U8_OPAQUE
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Converts into a premultiplied color.
    #[inline]
    pub fn premultiply(&self) -> PremultipliedColorU8 {
        let a = self.alpha();
        if a != ALPHA_U8_OPAQUE {
            PremultipliedColorU8::from_rgba_unchecked(
                premultiply_u8(self.red(), a),
                premultiply_u8(self.green(), a),
                premultiply_u8(self.blue(), a),
                a,
            )
        } else {
            PremultipliedColorU8::from_rgba_unchecked(self.red(), self.green(), self.blue(), a)
        }
    }
}

impl std::fmt::Debug for ColorU8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorU8")
            .field("r", &self.red())
            .field("g", &self.green())
            .field("b", &self.blue())
            .field("a", &self.alpha())
            .finish()
    }
}


/// A 32-bit premultiplied RGBA color value.
///
/// Byteorder: ABGR
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct PremultipliedColorU8(u32);

impl PremultipliedColorU8 {
    /// A transparent color.
    pub const TRANSPARENT: Self = PremultipliedColorU8::from_rgba_unchecked(0, 0, 0, 0);

    /// Creates a new color.
    #[inline]
    pub(crate) const fn from_rgba_unchecked(r: u8, g: u8, b: u8, a: u8) -> Self {
        PremultipliedColorU8(pack_rgba(r, g, b, a))
    }

    /// Returns color's red component.
    ///
    /// The value is <= alpha.
    #[inline]
    pub const fn red(self) -> u8 {
        ((self.0 >> 0) & 0xFF) as u8
    }

    /// Returns color's green component.
    ///
    /// The value is <= alpha.
    #[inline]
    pub const fn green(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// Returns color's blue component.
    ///
    /// The value is <= alpha.
    #[inline]
    pub const fn blue(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// Returns color's alpha component.
    #[inline]
    pub const fn alpha(self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    /// Check that color is opaque.
    ///
    /// Alpha == 255
    #[inline]
    pub fn is_opaque(&self) -> bool {
        self.alpha() == ALPHA_U8_OPAQUE
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns a demultiplied color.
    #[inline]
    pub fn demultiply(&self) -> ColorU8 {
        let a = self.alpha();
        if a == ALPHA_U8_OPAQUE {
            ColorU8::from_rgba(
                self.red(),
                self.green(),
                self.blue(),
                self.alpha(),
            )
        } else {
            self.to_color().demultiply().to_color_u8()
        }
    }

    #[inline]
    pub(crate) fn to_color(&self) -> PremultipliedColor {
        PremultipliedColor {
            r: normalize_u8(self.red()),
            g: normalize_u8(self.green()),
            b: normalize_u8(self.blue()),
            a: normalize_u8(self.alpha()),
        }
    }
}

impl std::fmt::Debug for PremultipliedColorU8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PremultipliedColorU8")
            .field("r", &self.red())
            .field("g", &self.green())
            .field("b", &self.blue())
            .field("a", &self.alpha())
            .finish()
    }
}


/// RGBA color value, holding four floating point components.
///
/// The container guarantees that all components are in a 0..=1 range.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    r: NormalizedF32,
    g: NormalizedF32,
    b: NormalizedF32,
    a: NormalizedF32,
}

const NV_ZERO: NormalizedF32 = NormalizedF32::ZERO;
const NV_ONE: NormalizedF32  = NormalizedF32::ONE;

impl Color {
    /// A transparent color.
    pub const TRANSPARENT: Color    = Color { r: NV_ZERO, g: NV_ZERO, b: NV_ZERO, a: NV_ZERO };
    /// A black color.
    pub const BLACK: Color          = Color { r: NV_ZERO, g: NV_ZERO, b: NV_ZERO, a: NV_ONE };
    /// A white color.
    pub const WHITE: Color          = Color { r: NV_ONE, g: NV_ONE, b: NV_ONE, a: NV_ONE };

    /// Creates a new color from 4 components.
    ///
    /// All values must be in 0..=1 range.
    #[inline]
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Option<Self> {
        Some(Color {
            r: NormalizedF32::new(r)?,
            g: NormalizedF32::new(g)?,
            b: NormalizedF32::new(b)?,
            a: NormalizedF32::new(a)?,
        })
    }

    /// Creates a new color from 4 components.
    #[inline]
    pub const fn from_rgba_safe(r: NormalizedF32, g: NormalizedF32, b: NormalizedF32, a: NormalizedF32) -> Self {
        Color { r, g, b, a }
    }

    /// Creates a new color from 4 components.
    ///
    /// u8 will be divided by 255 to get the float component.
    #[inline]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: normalize_u8(r),
            g: normalize_u8(g),
            b: normalize_u8(b),
            a: normalize_u8(a),
        }
    }

    /// Returns color's red component.
    ///
    /// The value is guarantee to be in a 0..=1 range.
    #[inline]
    pub fn red(&self) -> f32 {
        self.r.get()
    }

    /// Returns color's green component.
    ///
    /// The value is guarantee to be in a 0..=1 range.
    #[inline]
    pub fn green(&self) -> f32 {
        self.g.get()
    }

    /// Returns color's blue component.
    ///
    /// The value is guarantee to be in a 0..=1 range.
    #[inline]
    pub fn blue(&self) -> f32 {
        self.b.get()
    }

    /// Returns color's alpha component.
    ///
    /// The value is guarantee to be in a 0..=1 range.
    #[inline]
    pub fn alpha(&self) -> f32 {
        self.a.get()
    }

    /// Returns color's red component.
    #[inline]
    pub fn red_safe(&self) -> NormalizedF32 {
        self.r
    }

    /// Returns color's green component.
    #[inline]
    pub fn green_safe(&self) -> NormalizedF32 {
        self.g
    }

    /// Returns color's blue component.
    #[inline]
    pub fn blue_safe(&self) -> NormalizedF32 {
        self.b
    }

    /// Returns color's alpha component.
    #[inline]
    pub fn alpha_safe(&self) -> NormalizedF32 {
        self.a
    }

    /// Check that color is opaque.
    ///
    /// Alpha == 1.0
    #[inline]
    pub fn is_opaque(&self) -> bool {
        self.a == ALPHA_OPAQUE
    }

    /// Converts into a premultiplied color.
    #[inline]
    pub fn premultiply(&self) -> PremultipliedColor {
        if self.is_opaque() {
            PremultipliedColor {
                r: self.r,
                g: self.g,
                b: self.b,
                a: self.a,
            }
        } else {
            PremultipliedColor {
                r: NormalizedF32::new_bounded(self.r.get() * self.a.get()),
                g: NormalizedF32::new_bounded(self.g.get() * self.a.get()),
                b: NormalizedF32::new_bounded(self.b.get() * self.a.get()),
                a: self.a,
            }
        }
    }

    /// Converts into `ColorU8`.
    #[inline]
    pub fn to_color_u8(&self) -> ColorU8 {
        let c = color_f32_to_u8(self.r, self.g, self.b, self.a);
        ColorU8::from_rgba(c[0], c[1], c[2], c[3])
    }
}


/// Premultiplied RGBA color value, holding four floating point components.
///
/// The container guarantees that all components are in a 0..=1 range.
/// And RGB components are <= A.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PremultipliedColor {
    r: NormalizedF32,
    g: NormalizedF32,
    b: NormalizedF32,
    a: NormalizedF32,
}

impl PremultipliedColor {
    /// Returns color's red component.
    ///
    /// - The value is guarantee to be in a 0..=1 range.
    /// - The value is <= alpha.
    #[inline]
    pub fn red(&self) -> f32 {
        self.r.get()
    }

    /// Returns color's green component.
    ///
    /// - The value is guarantee to be in a 0..=1 range.
    /// - The value is <= alpha.
    #[inline]
    pub fn green(&self) -> f32 {
        self.g.get()
    }

    /// Returns color's blue component.
    ///
    /// - The value is guarantee to be in a 0..=1 range.
    /// - The value is <= alpha.
    #[inline]
    pub fn blue(&self) -> f32 {
        self.b.get()
    }

    /// Returns color's alpha component.
    ///
    /// - The value is guarantee to be in a 0..=1 range.
    #[inline]
    pub fn alpha(&self) -> f32 {
        self.a.get()
    }

    /// Returns color's red component.
    #[inline]
    pub fn red_safe(&self) -> NormalizedF32 {
        self.r
    }

    /// Returns color's green component.
    #[inline]
    pub fn green_safe(&self) -> NormalizedF32 {
        self.g
    }

    /// Returns color's blue component.
    #[inline]
    pub fn blue_safe(&self) -> NormalizedF32 {
        self.b
    }

    /// Returns color's alpha component.
    #[inline]
    pub fn alpha_safe(&self) -> NormalizedF32 {
        self.a
    }

    /// Returns a demultiplied color.
    #[inline]
    pub fn demultiply(&self) -> Color {
        unsafe {
            let a = self.a.get();
            if a == 0.0 {
                Color::TRANSPARENT
            } else {
                Color {
                    r: NormalizedF32::new_unchecked(self.r.get() / a),
                    g: NormalizedF32::new_unchecked(self.g.get() / a),
                    b: NormalizedF32::new_unchecked(self.b.get() / a),
                    a: self.a,
                }
            }
        }
    }

    /// Converts into `PremultipliedColorU8`.
    #[inline]
    pub fn to_color_u8(&self) -> PremultipliedColorU8 {
        let c = color_f32_to_u8(self.r, self.g, self.b, self.a);
        PremultipliedColorU8::from_rgba_unchecked(c[0], c[1], c[2], c[3])
    }
}

#[inline]
fn normalize_u8(n: u8) -> NormalizedF32 {
    unsafe {
        NormalizedF32::new_unchecked(n as f32 / 255.0)
    }
}

/// Return a*b/255, rounding any fractional bits.
#[inline]
pub fn premultiply_u8(c: u8, a: u8) -> u8 {
    let prod = u32::from(c) * u32::from(a) + 128;
    ((prod + (prod >> 8)) >> 8) as u8
}

#[inline]
const fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | ((r as u32) << 0)
}

#[inline]
fn color_f32_to_u8(r: NormalizedF32, g: NormalizedF32, b: NormalizedF32, a: NormalizedF32) -> [u8; 4] {
    [
        (r.get() * 255.0 + 0.5) as u8,
        (g.get() * 255.0 + 0.5) as u8,
        (b.get() * 255.0 + 0.5) as u8,
        (a.get() * 255.0 + 0.5) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiply_u8() {
        assert_eq!(
            ColorU8::from_rgba(10, 20, 30, 40).premultiply(),
            PremultipliedColorU8::from_rgba_unchecked(2, 3, 5, 40)
        );
    }

    #[test]
    fn premultiply_u8_opaque() {
        assert_eq!(
            ColorU8::from_rgba(10, 20, 30, 255).premultiply(),
            PremultipliedColorU8::from_rgba_unchecked(10, 20, 30, 255)
        );
    }

    #[test]
    fn demultiply_u8() {
        assert_eq!(
            PremultipliedColorU8::from_rgba_unchecked(2, 3, 5, 40).demultiply(),
            ColorU8::from_rgba(13, 19, 32, 40)
        );
    }

    #[test]
    fn demultiply_u8_opaque() {
        assert_eq!(
            PremultipliedColorU8::from_rgba_unchecked(10, 20, 30, 255).demultiply(),
            ColorU8::from_rgba(10, 20, 30, 255)
        );
    }
}
