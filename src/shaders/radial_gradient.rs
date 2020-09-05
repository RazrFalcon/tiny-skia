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
use crate::raster_pipeline;

#[derive(Copy, Clone, Debug)]
struct FocalData {
    r1: f32, // r1 after mapping focal point to (0, 0)
}

impl FocalData {
    // The input r0, r1 are the radii when we map centers to {(0, 0), (1, 0)}.
    // We'll post concat matrix with our transformation matrix that maps focal point to (0, 0).
    fn new(r1: f32, ts: &mut Transform) -> Option<Self> {
        // The following transformations are just to accelerate the shader computation by saving
        // some arithmetic operations.

        if is_focal_on_circle(r1) {
            ts.post_scale(0.5, 0.5);
        } else {
            ts.post_scale(r1 / (r1 * r1 - 1.0), 1.0 / ((r1 * r1 - 1.0).abs()).sqrt());
        }

        Some(FocalData {
            r1,
        })
    }

    fn is_focal_on_circle(&self) -> bool {
        is_focal_on_circle(self.r1)
    }

    fn is_well_behaved(&self) -> bool {
        !self.is_focal_on_circle() && self.r1 > 1.0
    }
}

// Whether the focal point (0, 0) is on the end circle with center (1, 0) and radius r1. If
// this is true, it's as if an aircraft is flying at Mach 1 and all circles (soundwaves)
// will go through the focal point (aircraft). In our previous implementations, this was
// known as the edge case where the inside circle touches the outside circle (on the focal
// point). If we were to solve for t bruteforcely using a quadratic equation, this case
// implies that the quadratic equation degenerates to a linear equation.
fn is_focal_on_circle(r1: f32) -> bool {
    (1.0 - r1).is_nearly_zero()
}


/// A radial gradient shader.
///
/// This is not `SkRadialGradient` like in Skia, but rather `SkTwoPointConicalGradient`
/// without the start radius.
#[derive(Clone, Debug)]
pub struct RadialGradient {
    base: Gradient,
    center1: Point,
    center2: Point,
    radius: f32,
    focal_data: Option<FocalData>,
}

impl RadialGradient {
    /// Creates a new radial gradient shader.
    ///
    /// Unlike Skia, doesn't return an empty or solid color shader on error.
    /// Also, doesn't fallback to a single point radial gradient.
    ///
    /// Returns `None` when:
    ///
    /// - `points.len()` < 2
    /// - `radius` <= 0
    /// - `transform` is not invertible
    pub fn new(
        start: Point,
        end: Point,
        radius: f32,
        points: Vec<GradientStop>,
        mode: SpreadMode,
        transform: Transform,
    ) -> Option<Shader> {
        // From SkGradientShader::MakeTwoPointConical

        if radius < 0.0 || radius.is_nearly_zero() {
            return None;
        }

        if points.len() < 2 {
            return None;
        }

        transform.invert()?;

        let length = (end - start).length();
        if !length.is_finite() {
            return None;
        }

        if length.is_nearly_zero_within_tolerance(DEGENERATE_THRESHOLD) {
            // If the center positions are the same, then the gradient
            // is the radial variant of a 2 pt conical gradient,
            // an actual radial gradient (startRadius == 0),
            // or it is fully degenerate (startRadius == endRadius).

            let inv = radius.invert();
            let mut ts = Transform::from_translate(-start.x, -start.y)?;
            ts.post_scale(inv, inv);

            // We can treat this gradient as radial, which is faster. If we got here, we know
            // that endRadius is not equal to 0, so this produces a meaningful gradient
            Some(Shader::RadialGradient(RadialGradient {
                base: Gradient::new(points, mode, transform, ts),
                center1: start,
                center2: end,
                radius,
                focal_data: None,
            }))
        } else {
            // From SkTwoPointConicalGradient::Create
            let mut ts = Transform::from_poly_to_poly(
                start, end,
                Point::from_xy(0.0, 0.0), Point::from_xy(1.0, 0.0),
            )?;

            let d_center = (start - end).length();
            let focal_data = Some(FocalData::new(radius / d_center, &mut ts)?);

            Some(Shader::RadialGradient(RadialGradient {
                base: Gradient::new(points, mode, transform, ts),
                center1: start,
                center2: end,
                radius,
                focal_data,
            }))
        }
    }

    pub(crate) fn push_stages(&self, rec: StageRec) -> bool {
        self.base.push_stages(rec, &|rec, post_p| {
            if let Some(focal_data) = self.focal_data {
                // Unlike, we have only the Focal radial gradient type.

                let ctx = raster_pipeline::TwoPointConicalGradientCtx {
                    mask: [0; 4],
                    p0: 1.0 / focal_data.r1,
                };

                let ctx = rec.ctx_storage.push_context(ctx);

                if focal_data.is_focal_on_circle() {
                    rec.pipeline.push(raster_pipeline::Stage::XYTo2PtConicalFocalOnCircle);
                } else if focal_data.is_well_behaved() {
                    rec.pipeline.push_with_context(
                        raster_pipeline::Stage::XYTo2PtConicalWellBehaved, ctx);
                } else {
                    rec.pipeline.push_with_context(
                        raster_pipeline::Stage::XYTo2PtConicalGreater, ctx);
                }

                if !focal_data.is_well_behaved() {
                    rec.pipeline.push_with_context(
                        raster_pipeline::Stage::Mask2PtConicalDegenerates, ctx);

                    post_p.push_with_context(raster_pipeline::Stage::ApplyVectorMask, ctx);
                }
            } else {
                rec.pipeline.push(raster_pipeline::Stage::XYToRadius);
            }
        }).is_some()
    }
}
