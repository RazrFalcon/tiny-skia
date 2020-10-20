// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Shader, Transform, Pixmap, SpreadMode, NormalizedF32};

use crate::shaders::StageRec;
use crate::pipeline;


/// Controls how much filtering to be done when transforming images.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FilterQuality {
    /// Nearest-neighbor. Low quality, but fastest.
    Nearest,
    /// Bilinear.
    Bilinear,
    /// Bicubic. High quality, but slow.
    Bicubic,
}


/// A pattern shader.
///
/// Essentially a `SkImageShader`.
///
/// Unlike Skia, we do not support FilterQuality::Medium, because it involves
/// mipmap generation, which adds too much complexity.
#[derive(Clone, Debug)]
pub struct Pattern<'a> {
    pixmap: &'a Pixmap,
    quality: FilterQuality,
    spread_mode: SpreadMode,
    pub(crate) opacity: NormalizedF32,
    pub(crate) transform: Transform,
}

impl<'a> Pattern<'a> {
    /// Creates a new pattern shader.
    pub fn new(
        pixmap: &'a Pixmap,
        spread_mode: SpreadMode,
        quality: FilterQuality,
        opacity: NormalizedF32,
        transform: Transform,
    ) -> Shader {
        Shader::Pattern(Pattern {
            pixmap,
            spread_mode,
            quality,
            opacity,
            transform,
        })
    }

    pub(crate) fn push_stages(&self, rec: StageRec) -> Option<()> {
        let ts = self.transform.invert()?;

        rec.pipeline.push(pipeline::Stage::SeedShader);

        rec.pipeline.push_transform(ts, rec.ctx_storage);

        let ctx = pipeline::GatherCtx {
            pixels: self.pixmap.pixels().as_ptr(),
            pixels_len: self.pixmap.pixels().len(),
            width: self.pixmap.size().width_safe(),
            height: self.pixmap.size().height_safe(),
        };

        let mut quality = self.quality;

        if ts.is_identity() || ts.is_translate() {
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

        // TODO: minimizing scale via mipmap

        match quality {
            FilterQuality::Nearest => {
                let limit_x = pipeline::TileCtx {
                    scale: self.pixmap.width() as f32,
                    inv_scale: 1.0 / self.pixmap.width() as f32,
                };

                let limit_y = pipeline::TileCtx {
                    scale: self.pixmap.height() as f32,
                    inv_scale: 1.0 / self.pixmap.height() as f32,
                };

                let limit_x = rec.ctx_storage.push_context(limit_x);
                let limit_y = rec.ctx_storage.push_context(limit_y);
                let ctx = rec.ctx_storage.push_context(ctx);

                match self.spread_mode {
                    SpreadMode::Pad => { /* The gather_xxx stage will clamp for us. */ }
                    SpreadMode::Repeat => {
                        rec.pipeline.push_with_context(pipeline::Stage::RepeatX, limit_x);
                        rec.pipeline.push_with_context(pipeline::Stage::RepeatY, limit_y);
                    }
                    SpreadMode::Reflect => {
                        rec.pipeline.push_with_context(pipeline::Stage::ReflectX, limit_x);
                        rec.pipeline.push_with_context(pipeline::Stage::ReflectY, limit_y);
                    }
                }

                rec.pipeline.push_with_context(pipeline::Stage::Gather, ctx);
            }
            FilterQuality::Bilinear => {
                let sampler_ctx = pipeline::SamplerCtx {
                    gather: ctx,
                    spread_mode: self.spread_mode,
                    inv_width: 1.0 / ctx.width.get() as f32,
                    inv_height: 1.0 / ctx.height.get() as f32,
                };
                let sampler_ctx = rec.ctx_storage.push_context(sampler_ctx);
                rec.pipeline.push_with_context(pipeline::Stage::Bilinear, sampler_ctx);
            }
            FilterQuality::Bicubic => {
                let sampler_ctx = pipeline::SamplerCtx {
                    gather: ctx,
                    spread_mode: self.spread_mode,
                    inv_width: 1.0 / ctx.width.get() as f32,
                    inv_height: 1.0 / ctx.height.get() as f32,
                };
                let sampler_ctx = rec.ctx_storage.push_context(sampler_ctx);
                rec.pipeline.push_with_context(pipeline::Stage::Bicubic, sampler_ctx);

                // Bicubic filtering naturally produces out of range values on both sides of [0,1].
                rec.pipeline.push(pipeline::Stage::Clamp0);
                rec.pipeline.push(pipeline::Stage::ClampA);
            }
        }

        // Unlike Skia, we do not support global opacity and only Pattern allows it.
        if self.opacity != NormalizedF32::ONE {
            debug_assert_eq!(std::mem::size_of_val(&self.opacity), 4, "alpha must be f32");
            let opacity = rec.ctx_storage.push_context(self.opacity.get());
            rec.pipeline.push_with_context(pipeline::Stage::Scale1Float, opacity);
        }

        Some(())
    }
}
