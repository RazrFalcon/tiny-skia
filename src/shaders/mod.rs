// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod gradient;
mod linear_gradient;
mod radial_gradient;

pub use gradient::GradientStop;
pub use linear_gradient::LinearGradient;
pub use radial_gradient::RadialGradient;

use crate::Color;

use crate::raster_pipeline::{RasterPipelineBuilder, ContextStorage};

#[allow(missing_debug_implementations)]
pub struct StageRec<'a> {
    pub(crate) ctx_storage: &'a mut ContextStorage,
    pub(crate) pipeline: &'a mut RasterPipelineBuilder,
}

/// A shader specifies the source color(s) for what is being drawn.
///
/// If a paint has no shader, then the paint's color is used. If the paint has a
/// shader, then the shader's color(s) are use instead, but they are
/// modulated by the paint's alpha. This makes it easy to create a shader
/// once (e.g. bitmap tiling or gradient) and then change its transparency
/// without having to modify the original shader. Only the paint's alpha needs
/// to be modified.
#[derive(Clone, Debug)]
pub enum Shader {
    /// A solid color shader.
    SolidColor(Color),
    /// A linear gradient shader.
    LinearGradient(LinearGradient),
    /// A radial gradient shader.
    RadialGradient(RadialGradient),
}

impl Shader {
    /// Checks if the shader is guaranteed to produce only opaque colors.
    #[inline]
    pub(crate) fn is_opaque(&self) -> bool {
        match self {
            Shader::SolidColor(ref c) => c.is_opaque(),
            Shader::LinearGradient(ref g) => g.is_opaque(),
            Shader::RadialGradient(_) => false,
        }
    }

    // Unlike Skia, we do not have is_constant, because we don't have Color shaders.

    /// If this returns false, then we draw nothing (do not fall back to shader context)
    pub(crate) fn push_stages(&self, rec: StageRec) -> bool {
        match self {
            Shader::SolidColor(_) => true,
            Shader::LinearGradient(ref g) => g.push_stages(rec),
            Shader::RadialGradient(ref g) => g.push_stages(rec),
        }
    }
}
