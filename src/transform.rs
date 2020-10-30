// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use num_ext::{FiniteF32, NonZeroF32};

use crate::Point;

use crate::scalar::{SCALAR_NEARLY_ZERO, Scalar};

// FiniteF32::default() is not `const` yet.
const FINITE_ZERO: FiniteF32 = unsafe { FiniteF32::new_unchecked(0.0) };
const NONZERO_ONE: NonZeroF32 = unsafe { NonZeroF32::new_unchecked(1.0) };

#[derive(Copy, Clone, PartialEq, Default)]
struct TransformFlags(u8);

impl TransformFlags {
    const IDENTITY: Self    = TransformFlags(0x00);
    const TRANSLATE: Self   = TransformFlags(0x01);
    const SCALE: Self       = TransformFlags(0x02);
    const SKEW: Self        = TransformFlags(0x04);

    #[inline] fn has_translate(self) -> bool { self.0 & 0x01 != 0 }
    #[inline] fn has_scale(self) -> bool { self.0 & 0x02 != 0 }
    #[inline] fn has_skew(self) -> bool { self.0 & 0x04 != 0 }
}

impl std::ops::BitOr for TransformFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self::Output {
        TransformFlags(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for TransformFlags {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0
    }
}


/// An affine transformation matrix.
///
/// Stores scale, skew and transform coordinates and a type of a transform.
///
/// # Guarantees
///
/// - All values are finite.
/// - ScaleX and/or ScaleY are not zero.
#[derive(Copy, Clone)]
pub struct Transform {
    sx: NonZeroF32, kx: FiniteF32,  tx: FiniteF32,
    ky: FiniteF32,  sy: NonZeroF32, ty: FiniteF32,
    flags: TransformFlags,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Transform {
            sx: NONZERO_ONE,
            kx: FINITE_ZERO,
            ky: FINITE_ZERO,
            sy: NONZERO_ONE,
            tx: FINITE_ZERO,
            ty: FINITE_ZERO,
            flags: TransformFlags::IDENTITY,
        }
    }
}

impl Transform {
    /// Creates an identity transform.
    #[inline]
    pub fn identity() -> Self {
        Transform::default()
    }

    /// Creates a new `Transform`.
    ///
    /// We are using column-major-column-vector matrix notation, therefore it's ky-kx, not kx-ky.
    ///
    /// # Checks
    ///
    /// - All values must be finite.
    /// - `sx` and `sy` must not be zero.
    #[inline]
    pub fn from_row(sx: f32, ky: f32, kx: f32, sy: f32, tx: f32, ty: f32) -> Option<Self> {
        let sx = NonZeroF32::new(sx)?;
        let ky = FiniteF32::new(ky)?;
        let kx = FiniteF32::new(kx)?;
        let sy = NonZeroF32::new(sy)?;
        let tx = FiniteF32::new(tx)?;
        let ty = FiniteF32::new(ty)?;
        Some(Transform::from_row_safe(sx, ky, kx, sy, tx, ty))
    }

    /// Creates a new `Transform`.
    ///
    /// We are using column-major-column-vector matrix notation, therefore it's ky-kx, not kx-ky.
    #[inline]
    pub(crate) fn from_row_safe(
        sx: NonZeroF32,
        ky: FiniteF32,
        kx: FiniteF32,
        sy: NonZeroF32,
        tx: FiniteF32,
        ty: FiniteF32,
    ) -> Self {
        let mut m = Transform {
            sx, kx, tx,
            ky, sy, ty,
            flags: TransformFlags::IDENTITY,
        };
        m.compute_flags();
        m
    }

    /// Creates a new `Transform` without checking values.
    ///
    /// # Safety
    ///
    /// - All values must be finite.
    /// - `sx` and `sy` must be non-zero.
    #[inline]
    pub unsafe fn from_row_unchecked(sx: f32, ky: f32, kx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        let sx = NonZeroF32::new_unchecked(sx);
        let ky = FiniteF32::new_unchecked(ky);
        let kx = FiniteF32::new_unchecked(kx);
        let sy = NonZeroF32::new_unchecked(sy);
        let tx = FiniteF32::new_unchecked(tx);
        let ty = FiniteF32::new_unchecked(ty);
        Transform::from_row_safe(sx, ky, kx, sy, tx, ty)
    }

    /// Creates a new translating Transform.
    ///
    /// # Checks
    ///
    /// - All values must be finite.
    #[inline]
    pub fn from_translate(tx: f32, ty: f32) -> Option<Self> {
        let tx = FiniteF32::new(tx)?;
        let ty = FiniteF32::new(ty)?;
        Some(Transform::from_translate_safe(tx, ty))
    }

    /// Creates a new translating Transform.
    #[inline]
    pub(crate) fn from_translate_safe(tx: FiniteF32, ty: FiniteF32) -> Self {
        let flags = if tx != FINITE_ZERO || ty != FINITE_ZERO {
            TransformFlags::TRANSLATE
        } else {
            TransformFlags::IDENTITY
        };

        Transform {
            sx: NONZERO_ONE, kx: FINITE_ZERO, tx,
            ky: FINITE_ZERO, sy: NONZERO_ONE, ty,
            flags,
        }
    }

    /// Creates a new scaling Transform.
    ///
    /// # Checks
    ///
    /// - All values must be finite.
    /// - `sx` and `sy` must not be zero.
    #[inline]
    pub fn from_scale(sx: f32, sy: f32) -> Option<Self> {
        let sx = NonZeroF32::new(sx)?;
        let sy = NonZeroF32::new(sy)?;
        Some(Transform::from_scale_safe(sx, sy))
    }

    /// Creates a new scaling Transform.
    #[inline]
    pub(crate) fn from_scale_safe(sx: NonZeroF32, sy: NonZeroF32) -> Self {
        let flags = if sx != NONZERO_ONE || sy != NONZERO_ONE {
            TransformFlags::SCALE
        } else {
            TransformFlags::IDENTITY
        };

        Transform {
            sx,              kx: FINITE_ZERO, tx: FINITE_ZERO,
            ky: FINITE_ZERO, sy,              ty: FINITE_ZERO,
            flags,
        }
    }

    /// Creates a new skewing Transform.
    ///
    /// # Checks
    ///
    /// - All values must be finite.
    #[inline]
    pub fn from_skew(kx: f32, ky: f32) -> Option<Self> {
        let kx = FiniteF32::new(kx)?;
        let ky = FiniteF32::new(ky)?;
        Some(Transform::from_skew_safe(kx, ky))
    }

    /// Creates a new skewing Transform.
    #[inline]
    pub(crate) fn from_skew_safe(kx: FiniteF32, ky: FiniteF32) -> Self {
        let flags = if kx != FINITE_ZERO || ky != FINITE_ZERO {
            TransformFlags::SKEW
        } else {
            TransformFlags::IDENTITY
        };

        Transform {
            sx: NONZERO_ONE, kx,              tx: FINITE_ZERO,
            ky,              sy: NONZERO_ONE, ty: FINITE_ZERO,
            flags,
        }
    }

    /// Returns scale pair.
    #[inline]
    pub fn get_scale(&self) -> (f32, f32) {
        (self.sx.get(), self.sy.get())
    }

    /// Returns skew pair.
    #[inline]
    pub fn get_skew(&self) -> (f32, f32) {
        (self.kx.get(), self.ky.get())
    }

    /// Returns translate pair.
    #[inline]
    pub fn get_translate(&self) -> (f32, f32) {
        (self.tx.get(), self.ty.get())
    }

    /// Returns all values.
    #[inline]
    pub fn get_row(&self) -> (f32, f32, f32, f32, f32, f32) {
        (self.sx.get(), self.ky.get(), self.kx.get(), self.sy.get(), self.tx.get(), self.ty.get())
    }

    /// Checks that transform is identity.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn is_identity(&self) -> bool {
        self.flags == TransformFlags::IDENTITY
    }

    /// Checks that transform is scale-only.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn is_scale(&self) -> bool {
        self.flags == TransformFlags::SCALE
    }

    /// Checks that transform is skew-only.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn is_skew(&self) -> bool {
        self.flags == TransformFlags::SKEW
    }

    /// Checks that transform is translate-only.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn is_translate(&self) -> bool {
        self.flags == TransformFlags::TRANSLATE
    }

    /// Checks that transform contains only scale and translate.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn is_scale_translate(&self) -> bool {
        self.flags == TransformFlags::SCALE ||
        self.flags == TransformFlags::TRANSLATE ||
        self.flags == TransformFlags::SCALE | TransformFlags::TRANSLATE
    }

    /// Checks that transform contains a scale part.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn has_scale(&self) -> bool {
        self.flags.has_scale()
    }

    /// Checks that transform contains a skew part.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn has_skew(&self) -> bool {
        self.flags.has_skew()
    }

    /// Checks that transform contains a translate part.
    ///
    /// The transform type is detected on creation, so this method is essentially free.
    #[inline]
    pub fn has_translate(&self) -> bool {
        self.flags.has_translate()
    }

    #[inline]
    fn compute_flags(&mut self) {
        self.flags = TransformFlags::IDENTITY;

        if self.sx != NONZERO_ONE || self.sy != NONZERO_ONE {
            self.flags |= TransformFlags::SCALE;
        }

        if self.tx != FINITE_ZERO || self.ty != FINITE_ZERO {
            self.flags |= TransformFlags::TRANSLATE;
        }

        if self.kx != FINITE_ZERO || self.ky != FINITE_ZERO {
            self.flags |= TransformFlags::SKEW;
        }
    }

    /// Pre-scales the current transform.
    #[inline]
    #[must_use]
    pub fn pre_scale(&self, sx: f32, sy: f32) -> Option<Self> {
        let other = Transform::from_scale(sx, sy)?;
        self.pre_concat(&other)
    }

    /// Post-scales the current transform.
    #[inline]
    #[must_use]
    pub fn post_scale(&mut self, sx: f32, sy: f32) -> Option<Self> {
        let other = Transform::from_scale(sx, sy)?;
        self.post_concat(&other)
    }

    /// Pre-translates the current transform.
    #[inline]
    #[must_use]
    pub fn pre_translate(&self, tx: f32, ty: f32) -> Option<Self> {
        let other = Transform::from_translate(tx, ty)?;
        self.pre_concat(&other)
    }

    /// Post-translates the current transform.
    #[inline]
    #[must_use]
    pub fn post_translate(&self, tx: f32, ty: f32) -> Option<Self> {
        let other = Transform::from_translate(tx, ty)?;
        self.post_concat(&other)
    }

    /// Pre-concats the current transform.
    #[inline]
    #[must_use]
    pub fn pre_concat(&self, other: &Self) -> Option<Self> {
        concat(self, other)
    }

    /// Post-concats the current transform.
    #[inline]
    #[must_use]
    pub fn post_concat(&self, other: &Self) -> Option<Self> {
        concat(other, self)
    }

    pub(crate) fn from_sin_cos(sin: f32, cos: f32) -> Option<Self> {
        Transform::from_row(cos, sin, -sin, cos, 0.0, 0.0)
    }

    pub(crate) fn map_points(&self, points: &mut [Point]) {
        if points.is_empty() {
            return;
        }

        // TODO: simd

        let (tx, ty) = self.get_translate();
        if self.is_identity() {
            // Do nothing.
        } else if self.is_translate() {
            for p in points {
                p.x += tx;
                p.y += ty;
            }
        } else if self.is_scale_translate() {
            let (sx, sy) = self.get_scale();
            for p in points {
                p.x = p.x * sx + tx;
                p.y = p.y * sy + ty;
            }
        } else {
            let (sx, sy) = self.get_scale();
            let (kx, ky) = self.get_skew();
            for p in points {
                let x = p.x * sx + p.y * kx + tx;
                let y = p.x * ky + p.y * sy + ty;
                p.x = x;
                p.y = y;
            }
        }
    }

    /// Returns an inverted transform.
    pub(crate) fn invert(&self) -> Option<Self> {
        // Allow the trivial case to be inlined.
        if self.is_identity() {
            return Some(*self);
        }

        invert(self)
    }

    pub(crate) fn to_unchecked(&self) -> TransformUnchecked {
        let (sx, ky, kx, sy, tx, ty) = self.get_row();
        TransformUnchecked { sx, ky, kx, sy, tx, ty }
    }
}

impl std::cmp::PartialEq for Transform {
    fn eq(&self, other: &Transform) -> bool {
        if self.flags != other.flags {
            false
        } else {
            self.sx == other.sx &&
            self.ky == other.ky &&
            self.kx == other.kx &&
            self.sy == other.sy &&
            self.tx == other.tx &&
            self.ty == other.ty
        }
    }
}

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transform")
            .field("sx", &self.sx)
            .field("ky", &self.ky)
            .field("kx", &self.kx)
            .field("sy", &self.sy)
            .field("tx", &self.tx)
            .field("ty", &self.ty)
            .finish()
    }
}

// Some of the Skia code relies on the fact that Transform/Matrix can have any values.
// In this cases we cannot use Transform.
// More specifically is that sx/sy can be zero.
#[derive(Copy, Clone, Debug)]
pub struct TransformUnchecked {
    pub sx: f32,
    pub ky: f32,
    pub kx: f32,
    pub sy: f32,
    pub tx: f32,
    pub ty: f32,
}

impl TransformUnchecked {
    pub fn from_translate(tx: f32, ty: f32) -> Self {
        TransformUnchecked::from_row(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    pub fn from_row(sx: f32, ky: f32, kx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        TransformUnchecked { sx, ky, kx, sy, tx, ty }
    }

    pub fn from_sin_cos_at(sin: f32, cos: f32, px: f32, py: f32) -> Self {
        let cos_inv = 1.0 - cos;
        TransformUnchecked::from_row(
            cos, sin, -sin, cos, sdot(sin, py, cos_inv, px), sdot(-sin, px, cos_inv, py)
        )
    }

    pub fn from_poly_to_poly(src1: Point, src2: Point, dst1: Point, dst2: Point) -> Option<Self> {
        let tmp = from_poly2(src1, src2);
        let res = tmp.to_safe()?.invert()?.to_unchecked();
        let tmp = from_poly2(dst1, dst2);
        let ts = concat_unchecked(&tmp, &res);
        Some(ts)
    }

    pub fn to_safe(&self) -> Option<Transform> {
        Transform::from_row(self.sx, self.ky, self.kx, self.sy, self.tx, self.ty)
    }

    pub fn is_identity(&self) -> bool {
        self.sx == 1.0 &&
        self.ky == 0.0 &&
        self.kx == 0.0 &&
        self.sy == 1.0 &&
        self.tx == 0.0 &&
        self.ty == 0.0
    }

    #[inline]
    #[must_use]
    pub fn post_translate(&self, tx: f32, ty: f32) -> Self {
        self.post_concat(&TransformUnchecked::from_translate(tx, ty))
    }

    #[inline]
    #[must_use]
    pub fn post_scale(&self, sx: f32, sy: f32) -> Self {
        self.post_concat(&TransformUnchecked::from_row(sx, 0.0, 0.0, sy, 0.0, 0.0))
    }

    #[inline]
    #[must_use]
    pub fn post_concat(&self, other: &Self) -> Self {
        concat_unchecked(other, self)
    }
}

fn from_poly2(p0: Point, p1: Point) -> TransformUnchecked {
    TransformUnchecked::from_row(
        p1.y - p0.y,
        p0.x - p1.x,
        p1.x - p0.x,
        p1.y - p0.y,
        p0.x,
        p0.y,
    )
}

fn concat_unchecked(a: &TransformUnchecked, b: &TransformUnchecked) -> TransformUnchecked {
    TransformUnchecked::from_row(
        mul_add_mul(a.sx, b.sx, a.kx, b.ky),
        mul_add_mul(a.ky, b.sx, a.sy, b.ky),
        mul_add_mul(a.sx, b.kx, a.kx, b.sy),
        mul_add_mul(a.ky, b.kx, a.sy, b.sy),
        mul_add_mul(a.sx, b.tx, a.kx, b.ty) + a.tx,
        mul_add_mul(a.ky, b.tx, a.sy, b.ty) + a.ty,
    )
}

#[inline(never)]
fn invert(ts: &Transform) -> Option<Transform> {
    debug_assert!(!ts.is_identity());

    if ts.is_scale_translate() {
        let (sx, _, _, sy, tx, ty) = ts.get_row();
        if ts.has_scale() {
            let inv_x = sx.invert();
            let inv_y = sy.invert();
            Transform::from_row(inv_x, 0.0, 0.0, inv_y, -tx * inv_x, -ty * inv_y)
        } else {
            // translate only
            Transform::from_translate(-tx, -ty)
        }
    } else {
        let inv_det = inv_determinant(ts)?;
        compute_inv(ts, inv_det)
    }
}

fn inv_determinant(ts: &Transform) -> Option<f64> {
    let (sx, ky, kx, sy, _, _) = ts.get_row();
    let det = dcross(sx as f64, sy as f64, kx as f64, ky as f64);

    // Since the determinant is on the order of the cube of the matrix members,
    // compare to the cube of the default nearly-zero constant (although an
    // estimate of the condition number would be better if it wasn't so expensive).
    let tolerance = SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO * SCALAR_NEARLY_ZERO;
    if (det as f32).is_nearly_zero_within_tolerance(tolerance) {
        None
    } else {
        Some(1.0 / det)
    }
}

fn compute_inv(ts: &Transform, inv_det: f64) -> Option<Transform> {
    let (sx, ky, kx, sy, tx, ty) = ts.get_row();

    Transform::from_row(
        (sy as f64 * inv_det) as f32,
        (-ky as f64 * inv_det) as f32,
        (-kx as f64 * inv_det) as f32,
        (sx as f64 * inv_det) as f32,
        dcross_dscale(
            kx,
            ty,
            sy,
            tx,
            inv_det,
        ),
        dcross_dscale(
            ky,
            tx,
            sx,
            ty,
            inv_det,
        ),
    )
}

fn dcross(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * b - c * d
}

fn dcross_dscale(a: f32, b: f32, c: f32, d: f32, scale: f64) -> f32 {
    (dcross(a as f64, b as f64, c as f64, d as f64) * scale) as f32
}

fn sdot(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * b + c * d
}

fn concat(a: &Transform, b: &Transform) -> Option<Transform> {
    if a.is_identity() {
        Some(*b)
    } else if b.is_identity() {
        Some(*a)
    } else if !a.has_skew() && !b.has_skew() {
        // just scale and translate
        let (a_sx, _, _, a_sy, a_tx, a_ty) = a.get_row();
        let (b_sx, _, _, b_sy, b_tx, b_ty) = b.get_row();
        Transform::from_row(
            a_sx * b_sx,
            0.0,
            0.0,
            a_sy * b_sy,
            a_sx * b_tx + a_tx,
            a_sy * b_ty + a_ty,
        )
    } else {
        let (a_sx, a_ky, a_kx, a_sy, a_tx, a_ty) = a.get_row();
        let (b_sx, b_ky, b_kx, b_sy, b_tx, b_ty) = b.get_row();
        Transform::from_row(
            mul_add_mul(a_sx, b_sx, a_kx, b_ky),
            mul_add_mul(a_ky, b_sx, a_sy, b_ky),
            mul_add_mul(a_sx, b_kx, a_kx, b_sy),
            mul_add_mul(a_ky, b_kx, a_sy, b_sy),
            mul_add_mul(a_sx, b_tx, a_kx, b_ty) + a_tx,
            mul_add_mul(a_ky, b_tx, a_sy, b_ty) + a_ty,
        )
    }
}

fn mul_add_mul(a: f32, b: f32, c: f32, d: f32) -> f32 {
    (f64::from(a) * f64::from(b) + f64::from(c) * f64::from(d)) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform() {
        assert_eq!(Transform::identity(),
                   Transform::from_row(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap());

        unsafe {
            assert_eq!(Transform::from_row(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).unwrap(),
                       Transform::from_row_unchecked(1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
        }

        assert_eq!(Transform::from_scale(1.0, 2.0).unwrap(),
                   Transform::from_row(1.0, 0.0, 0.0, 2.0, 0.0, 0.0).unwrap());

        assert_eq!(Transform::from_skew(2.0, 3.0).unwrap(),
                   Transform::from_row(1.0, 3.0, 2.0, 1.0, 0.0, 0.0).unwrap());

        assert_eq!(Transform::from_translate(5.0, 6.0).unwrap(),
                   Transform::from_row(1.0, 0.0, 0.0, 1.0, 5.0, 6.0).unwrap());

        assert_eq!(Transform::from_scale(0.0, 0.0), None);
        assert_eq!(Transform::from_scale(1.0, 0.0), None);
        assert_eq!(Transform::from_scale(0.0, 1.0), None);

        let ts = Transform::identity();
        assert_eq!(ts.is_identity(), true);
        assert_eq!(ts.is_scale(), false);
        assert_eq!(ts.is_skew(), false);
        assert_eq!(ts.is_translate(), false);
        assert_eq!(ts.is_scale_translate(), false);
        assert_eq!(ts.has_scale(), false);
        assert_eq!(ts.has_skew(), false);
        assert_eq!(ts.has_translate(), false);

        let ts = Transform::from_scale(2.0, 3.0).unwrap();
        assert_eq!(ts.is_identity(), false);
        assert_eq!(ts.is_scale(), true);
        assert_eq!(ts.is_skew(), false);
        assert_eq!(ts.is_translate(), false);
        assert_eq!(ts.is_scale_translate(), true);
        assert_eq!(ts.has_scale(), true);
        assert_eq!(ts.has_skew(), false);
        assert_eq!(ts.has_translate(), false);

        let ts = Transform::from_skew(2.0, 3.0).unwrap();
        assert_eq!(ts.is_identity(), false);
        assert_eq!(ts.is_scale(), false);
        assert_eq!(ts.is_skew(), true);
        assert_eq!(ts.is_translate(), false);
        assert_eq!(ts.is_scale_translate(), false);
        assert_eq!(ts.has_scale(), false);
        assert_eq!(ts.has_skew(), true);
        assert_eq!(ts.has_translate(), false);

        let ts = Transform::from_translate(2.0, 3.0).unwrap();
        assert_eq!(ts.is_identity(), false);
        assert_eq!(ts.is_scale(), false);
        assert_eq!(ts.is_skew(), false);
        assert_eq!(ts.is_translate(), true);
        assert_eq!(ts.is_scale_translate(), true);
        assert_eq!(ts.has_scale(), false);
        assert_eq!(ts.has_skew(), false);
        assert_eq!(ts.has_translate(), true);

        let ts = Transform::from_row(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).unwrap();
        assert_eq!(ts.is_identity(), false);
        assert_eq!(ts.is_scale(), false);
        assert_eq!(ts.is_skew(), false);
        assert_eq!(ts.is_translate(), false);
        assert_eq!(ts.is_scale_translate(), false);
        assert_eq!(ts.has_scale(), true);
        assert_eq!(ts.has_skew(), true);
        assert_eq!(ts.has_translate(), true);

        let ts = Transform::from_scale(1.0, 1.0).unwrap();
        assert_eq!(ts.has_scale(), false);

        let ts = Transform::from_skew(0.0, 0.0).unwrap();
        assert_eq!(ts.has_skew(), false);

        let ts = Transform::from_translate(0.0, 0.0).unwrap();
        assert_eq!(ts.has_translate(), false);
    }

    #[test]
    fn concat() {
        let mut ts = Transform::from_row(1.2, 3.4, -5.6, -7.8, 1.2, 3.4).unwrap();
        ts = ts.pre_scale(2.0, -4.0).unwrap();
        assert_eq!(ts, Transform::from_row(2.4, 6.8, 22.4, 31.2, 1.2, 3.4).unwrap());

        let mut ts = Transform::from_row(1.2, 3.4, -5.6, -7.8, 1.2, 3.4).unwrap();
        ts = ts.post_scale(2.0, -4.0).unwrap();
        assert_eq!(ts, Transform::from_row(2.4, -13.6, -11.2, 31.2, 2.4, -13.6).unwrap());
    }
}
