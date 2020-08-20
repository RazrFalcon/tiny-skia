// Copyright 2016 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::ffi::c_void;

use crate::{Paint, BlendMode, LengthU32, ScreenIntRect, Pixmap};

use crate::blitter::Blitter;
use crate::color::ALPHA_OPAQUE;
use crate::raster_pipeline::{self, RasterPipeline, RasterPipelineBuilder, ContextStorage};

pub struct RasterPipelineBlitter {
    blend: BlendMode,
    color_pipeline: RasterPipelineBuilder,

    dst_ctx: *const c_void, // Always points to the top-left of `pixmap`.

    // Built lazily on first use.
    blit_rect_rp: Option<RasterPipeline>,
}

impl RasterPipelineBlitter {
    fn new(
        paint: &Paint,
        shader_pipeline: &RasterPipelineBuilder,
        is_opaque: bool,
        _is_constant: bool,
        ctx_storage: &mut ContextStorage,
        pixmap: &mut Pixmap,
    ) -> Self {
        // Our job in this factory is to fill out the blitter's color pipeline.
        // This is the common front of the full blit pipelines, each constructed lazily on first use.
        // The full blit pipelines handle reading and writing the dst, blending, coverage, dithering.
        let mut color_pipeline = RasterPipelineBuilder::new();

        // Let's get the shader in first.
        color_pipeline.extend(shader_pipeline);

        // We can strength-reduce SrcOver into Src when opaque.
        let mut blend = paint.blend_mode;
        if is_opaque && blend == BlendMode::SourceOver {
            blend = BlendMode::Source;
        }

        let dst_ctx = ctx_storage.push_context(raster_pipeline::ffi::sk_raster_pipeline_memory_ctx {
            pixels: pixmap.data().as_ptr() as _,
            stride: pixmap.size().width().get() as i32,
        });

        // TODO: memset2D

        RasterPipelineBlitter {
            blend,
            color_pipeline,
            dst_ctx,
            blit_rect_rp: None,
        }
    }
}

impl Blitter for RasterPipelineBlitter {
    #[inline]
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        let one = unsafe { LengthU32::new_unchecked(1) };
        if let Some(r) = ScreenIntRect::from_nonzero_xywh(x, y, width, one) {
            self.blit_rect(r);
        }
    }

    fn blit_rect(&mut self, rect: ScreenIntRect) {
        if self.blit_rect_rp.is_none() {
            let mut p = RasterPipelineBuilder::new();
            p.extend(&self.color_pipeline);

            if self.blend == BlendMode::SourceOver {
                // TODO: ignore when dither_rate is non-zero
                p.push_with_context(raster_pipeline::Stage::SourceOverRgba8888, self.dst_ctx);
            } else {
                if self.blend != BlendMode::Source {
                    p.push_with_context(raster_pipeline::Stage::Load8888Destination, self.dst_ctx);
                    self.blend.push_stages(&mut p);
                }

                p.push_with_context(raster_pipeline::Stage::Store8888, self.dst_ctx);
            }

            self.blit_rect_rp = Some(p.compile());
        }

        self.blit_rect_rp.as_ref().unwrap().run(rect);
    }
}


pub fn create_raster_pipeline_blitter(
    paint: &Paint,
    ctx_storage: &mut ContextStorage,
    pixmap: &mut Pixmap,
) -> RasterPipelineBlitter {
    let mut shader_pipeline = RasterPipelineBuilder::new();

    // Having no shader makes things nice and easy... just use the paint color.
    let color_ctx = ctx_storage.create_uniform_color_context(paint.color.premultiply());
    shader_pipeline.push_with_context(raster_pipeline::Stage::UniformColor, color_ctx);

    let is_opaque = paint.color.alpha() == ALPHA_OPAQUE;
    let is_constant = true;
    RasterPipelineBlitter::new(paint, &shader_pipeline, is_opaque, is_constant, ctx_storage, pixmap)
}
