// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! tiny-skia specific checked-geom extensions.

use crate::{Point, Bounds, Transform};

use crate::scalar::{SCALAR_NEARLY_ZERO, Scalar};
use crate::wide::F32x4;


pub trait BoundsExt: Sized {
    fn from_points(points: &[Point]) -> Option<Self>;
}

impl BoundsExt for Bounds {
    /// Creates a Rect from Point array.
    ///
    /// Returns None if count is zero or if Point array contains an infinity or NaN.
    fn from_points(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut offset = 0;
        let mut min;
        let mut max;
        if points.len() & 1 != 0 {
            let pt = points[0];
            min = F32x4::new(pt.x, pt.y, pt.x, pt.y);
            max = min;
            offset += 1;
        } else {
            let pt0 = points[0];
            let pt1 = points[1];
            min = F32x4::new(pt0.x, pt0.y, pt1.x, pt1.y);
            max = min;
            offset += 2;
        }

        let mut accum = F32x4::default();
        while offset != points.len() {
            let pt0 = points[offset + 0];
            let pt1 = points[offset + 1];
            let xy = F32x4::new(pt0.x, pt0.y, pt1.x, pt1.y);

            accum *= xy;
            min = min.min(xy);
            max = max.max(xy);
            offset += 2;
        }

        // TODO: pathfinder's implementation of all_true is slightly different from the Skia. Test it.
        let all_finite = accum * F32x4::default() == F32x4::default();
        if all_finite {
            Bounds::from_ltrb(
                min.x().min(min.z()),
                min.y().min(min.w()),
                max.x().max(max.z()),
                max.y().max(max.w()),
            )
        } else {
            None
        }
    }
}


pub(crate) trait TransformExt: Sized {
    fn from_sin_cos(sin: f32, cos: f32) -> Option<Self>;
    fn from_sin_cos_at(sin: f32, cos: f32, px: f32, py: f32) -> Option<Self>;
    fn from_poly_to_poly(src1: Point, src2: Point, dst1: Point, dst2: Point) -> Option<Self>;
    fn pre_scale(&mut self, sx: f32, sy: f32);
    fn post_scale(&mut self, sx: f32, sy: f32);
    fn post_translate(&mut self, tx: f32, ty: f32);
    fn post_concat(&mut self, other: &Self);
    fn map_points(&self, points: &mut [Point]);
    fn invert(&self) -> Option<Self>;
    fn to_unchecked(&self) -> TransformUnchecked;
}

impl TransformExt for Transform {
    fn from_sin_cos(sin: f32, cos: f32) -> Option<Self> {
        Transform::from_row(cos, -sin, sin, cos, 0.0, 0.0)
    }

    fn from_sin_cos_at(sin: f32, cos: f32, px: f32, py: f32) -> Option<Self> {
        let cos_inv = 1.0 - cos;
        Transform::from_row(
            cos, -sin, sin, cos, sdot(sin, py, cos_inv, px), sdot(-sin, px, cos_inv, py)
        )
    }

    fn from_poly_to_poly(src1: Point, src2: Point, dst1: Point, dst2: Point) -> Option<Self> {
        let tmp = from_poly2(src1, src2);
        let res = tmp.to_safe()?.invert()?.to_unchecked();
        let tmp = from_poly2(dst1, dst2);
        let ts = concat_unchecked(&tmp, &res);
        ts.to_safe()
    }

    fn pre_scale(&mut self, sx: f32, sy: f32) {
        if sx == 1.0 && sy == 1.0 {
            return;
        }

        let (a_sx, a_kx, a_ky, a_sy, a_tx, a_ty) = self.get_row();
        *self = Transform::from_row(
            a_sx * sx,
            a_kx * sx,
            a_ky * sy,
            a_sy * sy,
            a_tx,
            a_ty,
        ).unwrap();
    }

    fn post_scale(&mut self, sx: f32, sy: f32) {
        if sx == 1.0 && sy == 1.0 {
            return;
        }

        // TODO: remove unwrap
        self.post_concat(&Transform::from_scale(sx, sy).unwrap())
    }

    fn post_translate(&mut self, dx: f32, dy: f32) {
        // TODO: can be optimized to just a sum
        let (sx, kx, ky, sy, tx, ty) = self.get_row();
        *self = Transform::from_row(sx, kx, ky, sy, tx + dx, ty + dy).unwrap();
    }

    fn post_concat(&mut self, other: &Self) {
        *self = concat(other, self).unwrap();
    }

    fn map_points(&self, points: &mut [Point]) {
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
                p.x = p.x * sx + p.y * kx + tx;
                p.y = p.x * ky + p.y * sy + ty;
            }
        }
    }

    fn invert(&self) -> Option<Self> {
        // Allow the trivial case to be inlined.
        if self.is_identity() {
            return Some(*self);
        }

        invert(self)
    }

    fn to_unchecked(&self) -> TransformUnchecked {
        let (sx, kx, ky, sy, tx, ty) = self.get_row();
        TransformUnchecked { sx, kx, ky, sy, tx, ty }
    }
}

// Some of the Skia code relies on the fact that Transform/Matrix can have any values.
// In this cases we cannot use Transform.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TransformUnchecked {
    sx: f32,
    kx: f32,
    ky: f32,
    sy: f32,
    tx: f32,
    ty: f32,
}

impl TransformUnchecked {
    fn from_row(sx: f32, kx: f32, ky: f32, sy: f32, tx: f32, ty: f32) -> Self {
        TransformUnchecked { sx, kx, ky, sy, tx, ty }
    }

    fn to_safe(&self) -> Option<Transform> {
        Transform::from_row(self.sx, self.kx, self.ky, self.sy, self.tx, self.ty)
    }
}

fn from_poly2(p0: Point, p1: Point) -> TransformUnchecked {
    TransformUnchecked::from_row(
        p1.y - p0.y,
        p1.x - p0.x,
        p0.x - p1.x,
        p1.y - p0.y,
        p0.x,
        p0.y,
    )
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
        let (a_sx, a_kx, a_ky, a_sy, a_tx, a_ty) = a.get_row();
        let (b_sx, b_kx, b_ky, b_sy, b_tx, b_ty) = b.get_row();
        Transform::from_row(
            mul_add_mul(a_sx, b_sx, a_kx, b_ky),
            mul_add_mul(a_sx, b_kx, a_kx, b_sy),
            mul_add_mul(a_ky, b_sx, a_sy, b_ky),
            mul_add_mul(a_ky, b_sx, a_sy, b_sy),
            mul_add_mul(a_sx, b_tx, a_kx, b_ty) + a_tx,
            mul_add_mul(a_ky, b_tx, a_sy, b_ty) + a_ty,
        )
    }
}

fn concat_unchecked(a: &TransformUnchecked, b: &TransformUnchecked) -> TransformUnchecked {
    TransformUnchecked::from_row(
        mul_add_mul(a.sx, b.sx, a.kx, b.ky),
        mul_add_mul(a.sx, b.kx, a.kx, b.sy),
        mul_add_mul(a.ky, b.sx, a.sy, b.ky),
        mul_add_mul(a.ky, b.sx, a.sy, b.sy),
        mul_add_mul(a.sx, b.tx, a.kx, b.ty) + a.tx,
        mul_add_mul(a.ky, b.tx, a.sy, b.ty) + a.ty,
    )
}

fn mul_add_mul(a: f32, b: f32, c: f32, d: f32) -> f32 {
    (f64::from(a) * f64::from(b) + f64::from(c) * f64::from(d)) as f32
}

#[inline(never)]
fn invert(ts: &Transform) -> Option<Transform> {
    debug_assert!(!ts.is_identity());

    if ts.is_scale_translate() {
        let (sx, _, _, sy, tx, ty) = ts.get_row();
        if ts.is_scale() {
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
    let (sx, kx, ky, sy, _, _) = ts.get_row();
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
    let (sx, kx, ky, sy, tx, ty) = ts.get_row();

    Transform::from_row(
        (sy as f64 * inv_det) as f32,
        (-kx as f64 * inv_det) as f32,
        (-ky as f64 * inv_det) as f32,
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
