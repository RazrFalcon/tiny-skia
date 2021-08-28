// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use crate::{Point, Shader, GradientStop, SpreadMode, Transform};

use crate::pipeline;
use crate::scalar::Scalar;
use crate::wide::u32x8;
use super::gradient::{Gradient, DEGENERATE_THRESHOLD};
use crate::pipeline::RasterPipelineBuilder;

#[cfg(all(not(feature = "std"), feature = "libm"))]
use crate::scalar::FloatExt;

#[derive(Copy, Clone, Debug)]
struct FocalData {
    r1: f32, // r1 after mapping focal point to (0, 0)
}

impl FocalData {
    // Whether the focal point (0, 0) is on the end circle with center (1, 0) and radius r1. If
    // this is true, it's as if an aircraft is flying at Mach 1 and all circles (soundwaves)
    // will go through the focal point (aircraft). In our previous implementations, this was
    // known as the edge case where the inside circle touches the outside circle (on the focal
    // point). If we were to solve for t bruteforcely using a quadratic equation, this case
    // implies that the quadratic equation degenerates to a linear equation.
    fn is_focal_on_circle(&self) -> bool {
        (1.0 - self.r1).is_nearly_zero()
    }

    fn is_well_behaved(&self) -> bool {
        !self.is_focal_on_circle() && self.r1 > 1.0
    }
}


/// A radial gradient shader.
///
/// This is not `SkRadialGradient` like in Skia, but rather `SkTwoPointConicalGradient`
/// without the start radius.
#[derive(Clone, Debug)]
pub struct RadialGradient {
    pub(crate) base: Gradient,
    center1: Point,
    center2: Point,
    radius: f32,
    focal_data: Option<FocalData>,
}

impl RadialGradient {
    /// Creates a new radial gradient shader.
    ///
    /// Returns `Shader::SolidColor` when:
    /// - `stops.len()` == 1
    ///
    /// Returns `None` when:
    ///
    /// - `stops` is empty
    /// - `radius` <= 0
    /// - `transform` is not invertible
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        start: Point,
        end: Point,
        radius: f32,
        stops: Vec<GradientStop>,
        mode: SpreadMode,
        transform: Transform,
    ) -> Option<Shader<'static>> {
        // From SkGradientShader::MakeTwoPointConical

        if radius < 0.0 || radius.is_nearly_zero() {
            return None;
        }

        if stops.is_empty() {
            return None;
        }

        if stops.len() == 1 {
            return Some(Shader::SolidColor(stops[0].color))
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
            let mut ts = Transform::from_translate(-start.x, -start.y);
            ts = ts.post_scale(inv, inv);

            // We can treat this gradient as radial, which is faster. If we got here, we know
            // that endRadius is not equal to 0, so this produces a meaningful gradient
            Some(Shader::RadialGradient(RadialGradient {
                base: Gradient::new(stops, mode, transform, ts),
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
            let r1 = radius / d_center;
            let focal_data = FocalData { r1 };

            // The following transformations are just to accelerate the shader computation by saving
            // some arithmetic operations.
            if focal_data.is_focal_on_circle() {
                ts = ts.post_scale(0.5, 0.5);
            } else {
                ts = ts.post_scale(r1 / (r1 * r1 - 1.0), 1.0 / ((r1 * r1 - 1.0).abs()).sqrt());
            }

            Some(Shader::RadialGradient(RadialGradient {
                base: Gradient::new(stops, mode, transform, ts),
                center1: start,
                center2: end,
                radius,
                focal_data: Some(focal_data),
            }))
        }
    }

    pub(crate) fn push_stages(&self, p: &mut RasterPipelineBuilder) -> Option<()> {
        let p0 = if let Some(focal_data) = self.focal_data {
            1.0 / focal_data.r1
        } else {
            1.0
        };

        p.ctx.two_point_conical_gradient = pipeline::TwoPointConicalGradientCtx {
            mask: u32x8::default(),
            p0,
        };

        self.base.push_stages(p,
            &|p| {
                if let Some(focal_data) = self.focal_data {
                    // Unlike Skia, we have only the Focal radial gradient type.

                    if focal_data.is_focal_on_circle() {
                        p.push(pipeline::Stage::XYTo2PtConicalFocalOnCircle);
                    } else if focal_data.is_well_behaved() {
                        p.push(pipeline::Stage::XYTo2PtConicalWellBehaved);
                    } else {
                        p.push(pipeline::Stage::XYTo2PtConicalGreater);
                    }

                    if !focal_data.is_well_behaved() {
                        p.push(pipeline::Stage::Mask2PtConicalDegenerates);
                    }
                } else {
                    p.push(pipeline::Stage::XYToRadius);
                }
            },
            &|p| {
                if let Some(focal_data) = self.focal_data {
                    if !focal_data.is_well_behaved() {
                        p.push(pipeline::Stage::ApplyVectorMask);
                    }
                }
            },
        )
    }
}
