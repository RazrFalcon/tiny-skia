// Copyright 2016 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::ffi::c_void;

use crate::{Paint, BlendMode, LengthU32, Pixmap, PremultipliedColorU8, Shader};
use crate::{ALPHA_U8_OPAQUE, ALPHA_U8_TRANSPARENT};

use crate::alpha_runs::AlphaRun;
use crate::blitter::{Blitter, Mask};
use crate::color::AlphaU8;
use crate::math::LENGTH_U32_ONE;
use crate::pipeline::{self, RasterPipeline, RasterPipelineBuilder, ContextStorage};
use crate::screen_int_rect::ScreenIntRect;
use crate::shaders::StageRec;


pub struct RasterPipelineBlitter<'a> {
    blend_mode: BlendMode,
    shader_pipeline: RasterPipelineBuilder,

    // Always points to the top-left of `pixmap`.
    pixels_ctx: pipeline::PixelsCtx<'a>,

    // Updated each call to blit_mask().
    mask_ctx: pipeline::MaskCtx,

    memset2d_color: Option<PremultipliedColorU8>,

    // Built lazily on first use.
    blit_anti_h_rp: Option<RasterPipeline>,
    blit_rect_rp: Option<RasterPipeline>,
    blit_mask_rp: Option<RasterPipeline>,

    // These values are pointed to by the blit pipelines above,
    // which allows us to adjust them from call to call.
    current_coverage: f32,
}

impl<'a> RasterPipelineBlitter<'a> {
    pub fn new(
        paint: &Paint,
        ctx_storage: &mut ContextStorage,
        pixmap: &'a mut Pixmap,
    ) -> Option<Self> {
        let mut shader_pipeline = RasterPipelineBuilder::new();
        shader_pipeline.force_hq_pipeline = paint.force_hq_pipeline;

        match &paint.shader {
            Shader::SolidColor(ref color) => {
                // Having no shader makes things nice and easy... just use the paint color.
                let color_ctx = ctx_storage.create_uniform_color_context(color.premultiply());
                shader_pipeline.push_with_context(pipeline::Stage::UniformColor, color_ctx);

                let is_constant = true;
                RasterPipelineBlitter::new_inner(paint, shader_pipeline, color.is_opaque(),
                                                 is_constant, pixmap)
            }
            shader => {
                let is_opaque = shader.is_opaque();
                let is_constant = false;

                let rec = StageRec {
                    ctx_storage,
                    pipeline: &mut shader_pipeline,
                };

                shader.push_stages(rec)?;
                RasterPipelineBlitter::new_inner(paint, shader_pipeline, is_opaque,
                                                 is_constant, pixmap)
            }
        }
    }

    fn new_inner(
        paint: &Paint,
        shader_pipeline: RasterPipelineBuilder,
        is_opaque: bool,
        is_constant: bool,
        pixmap: &'a mut Pixmap,
    ) -> Option<Self> {
        // Fast-reject.
        // This is basically SkInterpretXfermode().
        match paint.blend_mode {
            // `Destination` keep the pixmap unchanged. Nothing to do here.
            BlendMode::Destination => return None,
            BlendMode::DestinationIn if is_opaque && paint.is_solid_color() => return None,
            _ => {}
        }

        // Our job in this factory is to fill out the blitter's shader pipeline.
        // This is the common front of the full blit pipelines, each constructed lazily on first use.
        // The full blit pipelines handle reading and writing the dst, blending, coverage, dithering.

        // We can strength-reduce SrcOver into Src when opaque.
        let mut blend_mode = paint.blend_mode;
        if is_opaque && blend_mode == BlendMode::SourceOver {
            blend_mode = BlendMode::Source;
        }

        // When we're drawing a constant color in Source mode, we can sometimes just memset.
        let mut memset2d_color = None;
        if is_constant && blend_mode == BlendMode::Source {
            // Unlike Skia, our shader cannot be constant.
            // Therefore there is no need to run a raster pipeline to get shader's color.
            if let Shader::SolidColor(ref color) = paint.shader {
                memset2d_color = Some(color.premultiply().to_color_u8());
            }
        };

        // Clear is just a transparent color memset.
        if blend_mode == BlendMode::Clear {
            blend_mode = BlendMode::Source;
            memset2d_color = Some(PremultipliedColorU8::TRANSPARENT);
        }

        let img_ctx = pipeline::PixelsCtx {
            stride: pixmap.size().width_safe(),
            pixels: pixmap.pixels_mut(),
        };

        Some(RasterPipelineBlitter {
            blend_mode,
            shader_pipeline,
            pixels_ctx: img_ctx,
            mask_ctx: pipeline::MaskCtx::default(),
            memset2d_color,
            blit_anti_h_rp: None,
            blit_rect_rp: None,
            blit_mask_rp: None,
            current_coverage: 0.0,
        })
    }
}

impl Blitter for RasterPipelineBlitter<'_> {
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        let r = ScreenIntRect::from_xywh_safe(x, y, width, LENGTH_U32_ONE);
        self.blit_rect(&r);
    }

    fn blit_anti_h(&mut self, mut x: u32, y: u32, aa: &mut [AlphaU8], runs: &mut [AlphaRun]) {
        if self.blit_anti_h_rp.is_none() {
            let ctx_ptr = &self.pixels_ctx as *const _ as *const c_void;
            let curr_cov_ptr = &self.current_coverage as *const _ as *const c_void;

            let mut p = RasterPipelineBuilder::new();
            p.set_force_hq_pipeline(self.shader_pipeline.is_force_hq_pipeline());
            p.extend(&self.shader_pipeline);
            if self.blend_mode.should_pre_scale_coverage() {
                p.push_with_context(pipeline::Stage::Scale1Float, curr_cov_ptr);
                p.push_with_context(pipeline::Stage::LoadDestination, ctx_ptr);
                if let Some(blend_stage) = self.blend_mode.to_stage() {
                    p.push(blend_stage);
                }
            } else {
                p.push_with_context(pipeline::Stage::LoadDestination, ctx_ptr);
                if let Some(blend_stage) = self.blend_mode.to_stage() {
                    p.push(blend_stage);
                }

                p.push_with_context(pipeline::Stage::Lerp1Float, curr_cov_ptr);
            }

            p.push_with_context(pipeline::Stage::Store, ctx_ptr);

            self.blit_anti_h_rp = Some(p.compile());
        }

        let mut aa_offset = 0;
        let mut run_offset = 0;
        let mut run_opt = runs[0];
        while let Some(run) = run_opt {
            let width = LengthU32::from(run);

            match aa[aa_offset] {
                ALPHA_U8_TRANSPARENT => {}
                ALPHA_U8_OPAQUE => {
                    self.blit_h(x, y, width);
                }
                alpha => {
                    self.current_coverage = alpha as f32 * (1.0 / 255.0);

                    let rect = ScreenIntRect::from_xywh_safe(x, y, width, LENGTH_U32_ONE);
                    self.blit_anti_h_rp.as_ref().unwrap().run(&rect);
                }
            }

            x += width.get();
            run_offset += usize::from(run.get());
            aa_offset += usize::from(run.get());
            run_opt = runs[run_offset];
        }
    }

    fn blit_v(&mut self, x: u32, y: u32, height: LengthU32, alpha: AlphaU8) {
        let bounds = ScreenIntRect::from_xywh_safe(x, y, LENGTH_U32_ONE, height);

        let mask = Mask {
            image: [alpha, alpha],
            bounds,
            row_bytes: 0, // so we reuse the 1 "row" for all of height
        };

        self.blit_mask(&mask, &bounds);
    }

    fn blit_anti_h2(&mut self, x: u32, y: u32, alpha0: AlphaU8, alpha1: AlphaU8) {
        let bounds = ScreenIntRect::from_xywh(x, y, 2, 1).unwrap();

        let mask = Mask {
            image: [alpha0, alpha1],
            bounds,
            row_bytes: 2,
        };

        self.blit_mask(&mask, &bounds);
    }

    fn blit_anti_v2(&mut self, x: u32, y: u32, alpha0: AlphaU8, alpha1: AlphaU8) {
        let bounds = ScreenIntRect::from_xywh(x, y, 1, 2).unwrap();

        let mask = Mask {
            image: [alpha0, alpha1],
            bounds,
            row_bytes: 1,
        };

        self.blit_mask(&mask, &bounds);
    }

    fn blit_rect(&mut self, rect: &ScreenIntRect) {
        if let Some(c) = self.memset2d_color {
            for y in 0..rect.height() {
                let start = self.pixels_ctx.offset(rect.x() as usize, (rect.y() + y) as usize);
                let end = start + rect.width() as usize;
                self.pixels_ctx.pixels[start..end].iter_mut().for_each(|p| *p = c);
            }

            return;
        }

        if self.blit_rect_rp.is_none() {
            let mut p = RasterPipelineBuilder::new();
            p.set_force_hq_pipeline(self.shader_pipeline.is_force_hq_pipeline());
            p.extend(&self.shader_pipeline);

            let ctx_ptr = &self.pixels_ctx as *const _ as *const c_void;

            if self.blend_mode == BlendMode::SourceOver {
                // TODO: ignore when dither_rate is non-zero
                p.push_with_context(pipeline::Stage::SourceOverRgba, ctx_ptr);
            } else {
                if self.blend_mode != BlendMode::Source {
                    p.push_with_context(pipeline::Stage::LoadDestination, ctx_ptr);
                    if let Some(blend_stage) = self.blend_mode.to_stage() {
                        p.push(blend_stage);
                    }
                }

                p.push_with_context(pipeline::Stage::Store, ctx_ptr);
            }

            self.blit_rect_rp = Some(p.compile());
        }

        self.blit_rect_rp.as_ref().unwrap().run(rect);
    }

    fn blit_mask(&mut self, mask: &Mask, clip: &ScreenIntRect) {
        self.mask_ctx = pipeline::MaskCtx {
            pixels: mask.image,
            stride: mask.row_bytes,
            shift: (mask.bounds.left() + mask.bounds.top() * mask.row_bytes) as usize,
        };

        if self.blit_mask_rp.is_none() {
            let img_ctx_ptr = &self.pixels_ctx as *const _ as *const c_void;
            let mask_ctx_ptr = &self.mask_ctx as *const _ as *const c_void;

            let mut p = RasterPipelineBuilder::new();
            p.set_force_hq_pipeline(self.shader_pipeline.is_force_hq_pipeline());
            p.extend(&self.shader_pipeline);
            if self.blend_mode.should_pre_scale_coverage() {
                p.push_with_context(pipeline::Stage::ScaleU8, mask_ctx_ptr);
                p.push_with_context(pipeline::Stage::LoadDestination, img_ctx_ptr);
                if let Some(blend_stage) = self.blend_mode.to_stage() {
                    p.push(blend_stage);
                }
            } else {
                p.push_with_context(pipeline::Stage::LoadDestination, img_ctx_ptr);
                if let Some(blend_stage) = self.blend_mode.to_stage() {
                    p.push(blend_stage);
                }

                p.push_with_context(pipeline::Stage::LerpU8, mask_ctx_ptr);
            }

            p.push_with_context(pipeline::Stage::Store, img_ctx_ptr);

            self.blit_mask_rp = Some(p.compile());
        }

        self.blit_mask_rp.as_ref().unwrap().run(clip);
    }
}
