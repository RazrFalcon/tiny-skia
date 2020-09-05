// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Point, Shader, GradientStop, SpreadMode, Transform};

use crate::checked_geom_ext::TransformExt;
use crate::scalar::Scalar;
use super::gradient::{Gradient, DEGENERATE_THRESHOLD};
use crate::shaders::StageRec;

/// A linear gradient shader.
#[derive(Debug)]
pub struct LinearGradient {
    base: Gradient,
    start: Point,
    end: Point,
}

impl LinearGradient {
    /// Creates a new linear gradient shader.
    ///
    /// Unlike Skia, doesn't return an empty or solid color shader on error.
    ///
    /// Returns `None` when:
    ///
    /// - `points.len()` < 2
    /// - `start` == `end`
    /// - `transform` is not invertible
    pub fn new(
        start: Point,
        end: Point,
        points: Vec<GradientStop>,
        mode: SpreadMode,
        transform: Transform,
    ) -> Option<Box<dyn Shader>> {
        if points.len() < 2 {
            return None;
        }

        let length = (end - start).length();
        if !length.is_finite() {
            return None;
        }

        if length.is_nearly_zero_within_tolerance(DEGENERATE_THRESHOLD) {
            // Degenerate gradient, the only tricky complication is when in clamp mode, the limit of
            // the gradient approaches two half planes of solid color (first and last). However, they
            // are divided by the line perpendicular to the start and end point, which becomes undefined
            // once start and end are exactly the same, so just use the end color for a stable solution.
            //
            // Unlike Skia, we're not using `make_degenerate_gradient`.
            return None;
        }

        transform.invert()?;

        let unit_ts = points_to_unit_ts(start, end)?;
        Some(Box::new(LinearGradient {
            base: Gradient::new(points, mode, transform, unit_ts),
            start,
            end,
        }))
    }
}

impl Shader for LinearGradient {
    fn is_opaque(&self) -> bool { self.base.colors_are_opaque }

    fn push_stages(&self, rec: StageRec) -> bool {
        self.base.append_stages(rec, &|_, _| {}).is_some()
    }
}

fn points_to_unit_ts(start: Point, end: Point) -> Option<Transform> {
    let mut vec = end - start;
    let mag = vec.length();
    let inv = if mag != 0.0 { mag.invert() } else { 0.0 };

    vec.scale(inv);

    let mut ts = Transform::from_sin_cos_at(-vec.y, vec.x, start.x, start.y)?;
    ts.post_translate(-start.x, -start.y);
    ts.post_scale(inv, inv);
    Some(ts)
}
