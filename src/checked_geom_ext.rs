// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! tiny-skia specific checked-geom extensions.

use pathfinder_simd::default::{F32x2, F32x4};

use crate::{Point, Bounds};


pub trait PointExt: Sized {
    fn from_f32x2(r: F32x2) -> Self;
    fn to_f32x2(&self) -> F32x2;
}

impl PointExt for Point {
    #[inline]
    fn from_f32x2(r: F32x2) -> Self {
        Point::from_xy(r[0], r[1])
    }

    #[inline]
    fn to_f32x2(&self) -> F32x2 {
        F32x2::new(self.x, self.y)
    }
}


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

            accum = accum * xy;
            min = min.min(xy);
            max = max.max(xy);
            offset += 2;
        }

        // TODO: pathfinder's implementation of all_true is slightly different from the Skia. Test it.
        let all_finite = (accum * F32x4::default()).packed_eq(F32x4::default()).all_true();
        if all_finite {
            Bounds::from_ltrb(
                min[0].min(min[2]),
                min[1].min(min[3]),
                max[0].max(max[2]),
                max[1].max(max[3]),
            )
        } else {
            None
        }
    }
}
