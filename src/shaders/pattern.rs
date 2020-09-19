// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Shader,  Transform, FilterQuality, Pixmap};

use crate::safe_geom_ext::TransformExt;
use crate::shaders::StageRec;
use crate::raster_pipeline;

/// A pattern shader.
///
/// Similar to `SkImageShader`, but supports only the repeat mode.
/// Also, we do not support FilterQuality::Medium, because it involves
/// mipmap generation, which adds too much complexity.
#[derive(Clone, Debug)]
pub struct Pattern<'a> {
    pixmap: &'a Pixmap,
    quality: FilterQuality,
    transform: Transform,
}

impl<'a> Pattern<'a> {
    /// Creates a new pattern shader.
    pub fn new(
        pixmap: &'a Pixmap,
        quality: FilterQuality,
        transform: Transform,
    ) -> Shader {
        Shader::Pattern(Pattern {
            pixmap,
            quality,
            transform,
        })
    }

    pub(crate) fn push_stages(&self, rec: StageRec) -> Option<()> {
        let ts = self.transform.invert()?;

        rec.pipeline.push(raster_pipeline::Stage::SeedShader);

        rec.pipeline.push_transform(ts, rec.ctx_storage);

        let ctx = raster_pipeline::GatherCtx {
            pixels: self.pixmap.data().as_ptr() as _,
            stride: self.pixmap.size().width_safe(),
            width: self.pixmap.size().width_safe(),
            height: self.pixmap.size().height_safe(),
        };

        let limit_x = raster_pipeline::TileCtx {
            scale: self.pixmap.width() as f32,
            inv_scale: 1.0 / self.pixmap.width() as f32,
        };

        let limit_y = raster_pipeline::TileCtx {
            scale: self.pixmap.height() as f32,
            inv_scale: 1.0 / self.pixmap.height() as f32,
        };

        let limit_x = rec.ctx_storage.push_context(limit_x);
        let limit_y = rec.ctx_storage.push_context(limit_y);

        let mut quality = self.quality;

        if ts.is_identity() {
            quality = FilterQuality::Nearest;
        }

        if quality == FilterQuality::Bilinear {
            if ts.is_translate() {
                let (tx, ty) = ts.get_translate();
                if tx == tx.trunc() && ty == ty.trunc() {
                    // When the matrix is just an integer translate, bilerp == nearest neighbor.
                    quality = FilterQuality::Nearest;
                }
            }
        }

        // TODO: minimizing scale

        match quality {
            FilterQuality::Nearest => {
                let ctx = rec.ctx_storage.push_context(ctx);
                rec.pipeline.push_with_context(raster_pipeline::Stage::RepeatX, limit_x);
                rec.pipeline.push_with_context(raster_pipeline::Stage::RepeatY, limit_y);
                rec.pipeline.push_with_context(raster_pipeline::Stage::Gather, ctx);
            }
            FilterQuality::Bilinear => {
                let sampler_ctx = raster_pipeline::SamplerCtx {
                    gather: ctx,
                    inv_width: 1.0 / ctx.width.get() as f32,
                    inv_height: 1.0 / ctx.height.get() as f32,
                };
                let sampler_ctx = rec.ctx_storage.push_context(sampler_ctx);
                rec.pipeline.push_with_context(raster_pipeline::Stage::Bilinear, sampler_ctx);
            }
            FilterQuality::Bicubic => {
                let sampler_ctx = raster_pipeline::SamplerCtx {
                    gather: ctx,
                    inv_width: 1.0 / ctx.width.get() as f32,
                    inv_height: 1.0 / ctx.height.get() as f32,
                };
                let sampler_ctx = rec.ctx_storage.push_context(sampler_ctx);
                rec.pipeline.push_with_context(raster_pipeline::Stage::Bicubic, sampler_ctx);

                // Bicubic filtering naturally produces out of range values on both sides of [0,1].
                rec.pipeline.push(raster_pipeline::Stage::Clamp0);
                rec.pipeline.push(raster_pipeline::Stage::ClampA);
            }
        }

        Some(())
    }
}
