// Copyright 2016 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::ffi::c_void;

use crate::{Paint, BlendMode, LengthU32, ScreenIntRect, Pixmap, PremultipliedColorU8};

use crate::blitter::Blitter;
use crate::raster_pipeline::{self, RasterPipeline, RasterPipelineBuilder, ContextStorage};

pub fn create_raster_pipeline_blitter(
    paint: &Paint,
    ctx_storage: &mut ContextStorage,
    pixmap: &mut Pixmap,
) -> Option<RasterPipelineBlitter> {
    let mut shader_pipeline = RasterPipelineBuilder::new();

    // Having no shader makes things nice and easy... just use the paint color.
    let color_ctx = ctx_storage.create_uniform_color_context(paint.color.premultiply());
    shader_pipeline.push_with_context(raster_pipeline::Stage::UniformColor, color_ctx);

    RasterPipelineBlitter::new(paint, &shader_pipeline, pixmap)
}


pub struct RasterPipelineBlitter {
    blend_mode: BlendMode,
    color_pipeline: RasterPipelineBuilder,

    // Always points to the top-left of `pixmap`.
    img_ctx: raster_pipeline::ffi::sk_raster_pipeline_memory_ctx,

    memset2d_color: Option<PremultipliedColorU8>,

    // Built lazily on first use.
    blit_rect_rp: Option<RasterPipeline>,
}

impl RasterPipelineBlitter {
    fn new(
        paint: &Paint,
        shader_pipeline: &RasterPipelineBuilder,
        pixmap: &mut Pixmap,
    ) -> Option<Self> {
        // Fast-reject.
        // This is basically SkInterpretXfermode().
        match paint.blend_mode {
            // `Destination` keep the pixmap unchanged. Nothing to do here.
            BlendMode::Destination => return None,
            // TODO: disabled by shader
            BlendMode::DestinationIn if paint.color.is_opaque() => return None,
            _ => {}
        }

        // Our job in this factory is to fill out the blitter's color pipeline.
        // This is the common front of the full blit pipelines, each constructed lazily on first use.
        // The full blit pipelines handle reading and writing the dst, blending, coverage, dithering.
        let mut color_pipeline = RasterPipelineBuilder::new();

        // Let's get the shader in first.
        color_pipeline.extend(shader_pipeline);

        // We can strength-reduce SrcOver into Src when opaque.
        let is_opaque = paint.color.is_opaque(); // TODO: affected by shader
        let mut blend_mode = paint.blend_mode;
        if is_opaque && blend_mode == BlendMode::SourceOver {
            blend_mode = BlendMode::Source;
        }

        // When we're drawing a constant color in Source mode, we can sometimes just memset.
        let mut memset2d_color = None;
        if blend_mode == BlendMode::Source {
            // TODO: will be affected by a shader and dither later on
            memset2d_color = Some(paint.color.premultiply().to_color_u8());
        };

        // Clear is just a transparent color memset.
        if blend_mode == BlendMode::Clear {
            blend_mode = BlendMode::Source;
            memset2d_color = Some(PremultipliedColorU8::TRANSPARENT);
        }

        let dst_ctx = raster_pipeline::ffi::sk_raster_pipeline_memory_ctx {
            pixels: pixmap.data().as_ptr() as _,
            stride: pixmap.size().width().get() as i32,
        };

        Some(RasterPipelineBlitter {
            blend_mode,
            color_pipeline,
            img_ctx: dst_ctx,
            memset2d_color,
            blit_rect_rp: None,
        })
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
        // TODO: reject out of bounds access

        if let Some(c) = self.memset2d_color {
            for y in 0..rect.height().get() {
                // Cast pixmap data to color.
                let mut addr = self.img_ctx.pixels.cast::<PremultipliedColorU8>();

                // Calculate pixel offset in bytes.
                debug_assert!(self.img_ctx.stride > 0);
                let offset = calc_pixel_offset(rect.x(), rect.y() + y, self.img_ctx.stride as u32);
                addr = unsafe { addr.add(offset) };

                for _ in 0..rect.width().get() as usize {
                    unsafe {
                        *addr = c;
                        addr = addr.add(1);
                    }
                }
            }

            return;
        }

        if self.blit_rect_rp.is_none() {
            let mut p = RasterPipelineBuilder::new();
            p.extend(&self.color_pipeline);

            let ctx_ptr = &self.img_ctx as *const _ as *const c_void;

            if self.blend_mode == BlendMode::SourceOver {
                // TODO: ignore when dither_rate is non-zero
                p.push_with_context(raster_pipeline::Stage::SourceOverRgba8888, ctx_ptr);
            } else {
                if self.blend_mode != BlendMode::Source {
                    p.push_with_context(raster_pipeline::Stage::Load8888Destination, ctx_ptr);
                    self.blend_mode.push_stages(&mut p);
                }

                p.push_with_context(raster_pipeline::Stage::Store8888, ctx_ptr);
            }

            self.blit_rect_rp = Some(p.compile());
        }

        self.blit_rect_rp.as_ref().unwrap().run(rect);
    }
}

#[inline]
fn calc_pixel_offset(x: u32, y: u32, stride: u32) -> usize {
    calc_pixel_offset_usize(x as usize, y as usize, stride as usize)
}

#[inline]
fn calc_pixel_offset_usize(x: usize, y: usize, stride: usize) -> usize {
    y * stride + x
}
