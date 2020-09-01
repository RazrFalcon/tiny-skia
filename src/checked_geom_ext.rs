// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! tiny-skia specific checked-geom extensions.

use crate::{Point, Bounds, Transform};

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


pub trait TransformExt: Sized {
    fn from_sin_cos(sin: f32, cos: f32) -> Option<Self>;
    fn pre_scale(&mut self, sx: f32, sy: f32);
    fn post_concat(&mut self, other: &Self);
    fn map_points(&self, points: &mut [Point]);
}

impl TransformExt for Transform {
    #[inline]
    fn from_sin_cos(sin: f32, cos: f32) -> Option<Self> {
        Transform::from_row(cos, -sin, sin, cos, 0.0, 0.0)
    }

    #[inline]
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

    #[inline]
    fn post_concat(&mut self, other: &Self) {
        *self = concat(other, self);
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
}

fn concat(a: &Transform, b: &Transform) -> Transform {
    if a.is_identity() {
        *b
    } else if b.is_identity() {
        *a
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
        ).unwrap()
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
        ).unwrap()
    }
}

#[inline]
fn mul_add_mul(a: f32, b: f32, c: f32, d: f32) -> f32 {
    (f64::from(a) * f64::from(b) + f64::from(c) * f64::from(d)) as f32
}
