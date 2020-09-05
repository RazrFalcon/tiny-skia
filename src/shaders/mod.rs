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
pub trait Shader: std::fmt::Debug {
    /// Checks if the shader is guaranteed to produce only opaque colors.
    fn is_opaque(&self) -> bool { false }

    // Unlike Skia, we do not have is_constant, because we don't have Color shaders.

    #[doc(hide)]
    /// If this returns false, then we draw nothing (do not fall back to shader context)
    fn push_stages(&self, rec: StageRec) -> bool;
}
