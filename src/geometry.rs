// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use pathfinder_simd::default::F32x2;

use crate::Point;

mod private {
    #[derive(Copy, Clone, PartialEq, Debug)]
    #[repr(transparent)]
    pub struct TValue(f32);

    impl TValue {
        pub const HALF: Self = TValue(0.5);

        #[inline]
        pub fn new(n: f32) -> Option<Self> {
            if n.is_finite() && n > 0.0 && n < 1.0 {
                Some(TValue(n))
            } else {
                None
            }
        }

        #[inline]
        pub fn get(self) -> f32 {
            self.0
        }
    }
}
use private::TValue;

/// Returns 0 for 1 quad, and 1 for two quads, either way the answer is stored in dst[].
///
/// Guarantees that the 1/2 quads will be monotonic.
pub fn chop_quad_at_y_extrema(src: &[Point; 3], dst: &mut [Point; 5]) -> usize {
    let a = src[0].y;
    let mut b = src[1].y;
    let c = src[2].y;

    if is_not_monotonic(a, b, c) {
        if let Some(t_value) = valid_unit_divide(a - b, a - b - b + c) {
            chop_quad_at(src, dst, t_value);
            flatten_double_quad_extrema(dst);
            return 1;
        }

        // if we get here, we need to force dst to be monotonic, even though
        // we couldn't compute a unit_divide value (probably underflow).
        b = if (a - b).abs() < (b - c).abs() { a } else { c };
    }

    dst[0] = Point::from_xy(src[0].x, a);
    dst[1] = Point::from_xy(src[1].x, b);
    dst[2] = Point::from_xy(src[2].x, c);
    0
}

#[inline]
fn is_not_monotonic(a: f32, b: f32, c: f32) -> bool {
    let ab = a - b;
    let mut bc = b - c;
    if ab < 0.0 {
        bc = -bc;
    }

    ab == 0.0 || bc < 0.0
}

fn chop_quad_at(src: &[Point], dst: &mut [Point; 5], t: TValue) {
    let p0 = src[0].to_f32x2();
    let p1 = src[1].to_f32x2();
    let p2 = src[2].to_f32x2();
    let tt = F32x2::splat(t.get());

    let p01 = interp(p0, p1, tt);
    let p12 = interp(p1, p2, tt);

    dst[0] = Point::from_f32x2(p0);
    dst[1] = Point::from_f32x2(p01);
    dst[2] = Point::from_f32x2(interp(p01, p12, tt));
    dst[3] = Point::from_f32x2(p12);
    dst[4] = Point::from_f32x2(p2);
}

#[inline]
fn flatten_double_quad_extrema(coords: &mut [Point; 5]) {
    coords[1].y = coords[2].y;
    coords[3].y = coords[2].y;
}


/// Given 4 points on a cubic bezier, chop it into 1, 2, 3 beziers such that
/// the resulting beziers are monotonic in Y.
///
/// This is called by the scan converter.
///
/// Depending on what is returned, dst[] is treated as follows:
///
/// - 0: dst[0..3] is the original cubic
/// - 1: dst[0..3] and dst[3..6] are the two new cubics
/// - 2: dst[0..3], dst[3..6], dst[6..9] are the three new cubics
pub fn chop_cubic_at_y_extrema(src: &[Point; 4], dst: &mut [Point; 10]) -> usize {
    // We can use random numbers here, because they will be overwritten anyway.
    let mut t_values = [TValue::HALF, TValue::HALF];
    let t_values = find_cubic_extrema(src[0].y, src[1].y, src[2].y, src[3].y, &mut t_values);

    chop_cubic_at(src, &t_values, dst);
    if !t_values.is_empty() {
        // we do some cleanup to ensure our Y extrema are flat
        flatten_double_cubic_extrema(dst);
        if t_values.len() == 2 {
            flatten_double_cubic_extrema(&mut dst[3..]);
        }
    }

    t_values.len()
}

// Cubic'(t) = At^2 + Bt + C, where
// A = 3(-a + 3(b - c) + d)
// B = 6(a - 2b + c)
// C = 3(b - a)
// Solve for t, keeping only those that fit between 0 < t < 1
fn find_cubic_extrema(a: f32, b: f32, c: f32, d: f32, t_values: &mut [TValue; 2]) -> &[TValue] {
    // we divide A,B,C by 3 to simplify
    let na = d - a + 3.0 * (b - c);
    let nb = 2.0 * (a - b - b + c);
    let nc = b - a;

    let roots = find_unit_quad_roots(na, nb, nc, t_values);
    &t_values[0..roots]
}

/// From Numerical Recipes in C.
///
/// Q = -1/2 (B + sign(B) sqrt[B*B - 4*A*C])
/// x1 = Q / A
/// x2 = C / Q
fn find_unit_quad_roots(a: f32, b: f32, c: f32, roots: &mut [TValue; 2]) -> usize {
    debug_assert!(roots.len() >= 2);

    if a == 0.0 {
        if let Some(r) = valid_unit_divide(-c, b) {
            roots[0] = r;
            return 1;
        } else {
            return 0;
        }
    }

    // use doubles so we don't overflow temporarily trying to compute R
    let mut dr = f64::from(b) * f64::from(b) - 4.0 * f64::from(a) * f64::from(c);
    if dr < 0.0 {
        return 0;
    }
    dr = dr.sqrt();
    let r = dr as f32;
    if !r.is_finite() {
        return 0;
    }

    let q = if b < 0.0 { -(b - r) / 2.0 } else { -(b + r) / 2.0 };

    let mut roots_offset = 0;
    if let Some(r) = valid_unit_divide(q, a) {
        roots[roots_offset] = r;
        roots_offset += 1;
    }

    if let Some(r) = valid_unit_divide(c, q) {
        roots[roots_offset] = r;
        roots_offset += 1;
    }

    if roots_offset == 2 {
        if roots[0].get() > roots[1].get() {
            roots.swap(0, 1);
        } else if roots[0] == roots[1] { // nearly-equal?
            roots_offset -= 1; // skip the double root
        }
    }

    roots_offset
}

// http://code.google.com/p/skia/issues/detail?id=32
//
// This test code would fail when we didn't check the return result of
// valid_unit_divide in SkChopCubicAt(... tValues[], int roots). The reason is
// that after the first chop, the parameters to valid_unit_divide are equal
// (thanks to finite float precision and rounding in the subtracts). Thus
// even though the 2nd tValue looks < 1.0, after we renormalize it, we end
// up with 1.0, hence the need to check and just return the last cubic as
// a degenerate clump of 4 points in the same place.
//
// static void test_cubic() {
//     SkPoint src[4] = {
//         { 556.25000, 523.03003 },
//         { 556.23999, 522.96002 },
//         { 556.21997, 522.89001 },
//         { 556.21997, 522.82001 }
//     };
//     SkPoint dst[10];
//     SkScalar tval[] = { 0.33333334f, 0.99999994f };
//     SkChopCubicAt(src, dst, tval, 2);
// }
fn chop_cubic_at(src: &[Point; 4], t_values: &[TValue], dst: &mut [Point]) {
    if t_values.is_empty() {
        // nothing to chop
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
        dst[3] = src[3];
    } else {
        let t = t_values[0];
        let mut tmp = [Point::zero(); 4];

        // Reduce the `src` lifetime, so we can use `src = &tmp` later.
        let mut src = src;

        let mut dst_offset = 0;
        for i in 0..t_values.len() {
            chop_cubic_at2(src, t, dst);
            if i == t_values.len() - 1 {
                break;
            }

            dst_offset += 3;
            // have src point to the remaining cubic (after the chop)
            tmp[0] = dst[dst_offset + 0];
            tmp[1] = dst[dst_offset + 1];
            tmp[2] = dst[dst_offset + 2];
            tmp[3] = dst[dst_offset + 3];
            src = &tmp;

            // watch out in case the renormalized t isn't in range
            let n = valid_unit_divide(
                t_values[i+1].get() - t_values[i].get(),
                1.0 - t_values[i].get(),
            );
            if n.is_none() {
                // if we can't, just create a degenerate cubic
                dst[4] = src[3];
                dst[5] = src[3];
                dst[6] = src[3];
                break;
            }
        }
    }
}

fn chop_cubic_at2(src: &[Point; 4], t: TValue, dst: &mut [Point]) {
    let p0 = src[0].to_f32x2();
    let p1 = src[1].to_f32x2();
    let p2 = src[2].to_f32x2();
    let p3 = src[3].to_f32x2();
    let tt = F32x2::splat(t.get());

    let ab = interp(p0, p1, tt);
    let bc = interp(p1, p2, tt);
    let cd = interp(p2, p3, tt);
    let abc = interp(ab, bc, tt);
    let bcd = interp(bc, cd, tt);
    let abcd = interp(abc, bcd, tt);

    dst[0] = Point::from_f32x2(p0);
    dst[1] = Point::from_f32x2(ab);
    dst[2] = Point::from_f32x2(abc);
    dst[3] = Point::from_f32x2(abcd);
    dst[4] = Point::from_f32x2(bcd);
    dst[5] = Point::from_f32x2(cd);
    dst[6] = Point::from_f32x2(p3);
}

#[inline]
fn flatten_double_cubic_extrema(coords: &mut [Point]) {
    coords[2].y = coords[3].y;
    coords[4].y = coords[3].y;
}

#[inline]
fn valid_unit_divide(mut numer: f32, mut denom: f32) -> Option<TValue> {
    if numer < 0.0 {
        numer = -numer;
        denom = -denom;
    }

    if denom == 0.0 || numer == 0.0 || numer >= denom {
        return None;
    }

    let r = numer / denom;
    TValue::new(r)
}

#[inline]
fn interp(v0: F32x2, v1: F32x2, t: F32x2) -> F32x2 {
    v0 + (v1 - v0) * t
}
