// Copyright 2016 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::ffi::c_void;

use crate::{ScreenIntRect, LengthU32, Transform, Color, NormalizedF32};

use crate::color::PremultipliedColor;

pub use blitter::RasterPipelineBlitter;

mod blitter;
mod lowp;
mod highp;


#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum Stage {
    MoveSourceToDestination = 0,
    MoveDestinationToSource,
    Clamp0,
    ClampA,
    Premultiply,
    UniformColor,
    SeedShader,
    LoadDestination,
    Store,
    Gather,
    ScaleU8,
    LerpU8,
    Scale1Float,
    Lerp1Float,
    DestinationAtop,
    DestinationIn,
    DestinationOut,
    DestinationOver,
    SourceAtop,
    SourceIn,
    SourceOut,
    SourceOver,
    Clear,
    Modulate,
    Multiply,
    Plus,
    Screen,
    Xor,
    ColorBurn,
    ColorDodge,
    Darken,
    Difference,
    Exclusion,
    HardLight,
    Lighten,
    Overlay,
    SoftLight,
    Hue,
    Saturation,
    Color,
    Luminosity,
    SourceOverRgba,
    TransformTranslate, // TODO: remove?
    TransformScaleTranslate, // TODO: remove?
    Transform2X3,
    RepeatX,
    RepeatY,
    Bilinear,
    Bicubic,
    PadX1,
    ReflectX1,
    RepeatX1,
    Gradient,
    EvenlySpaced2StopGradient,
    XYToRadius,
    XYTo2PtConicalFocalOnCircle,
    XYTo2PtConicalWellBehaved,
    XYTo2PtConicalGreater,
    Mask2PtConicalDegenerates,
    ApplyVectorMask,
}

pub const STAGES_COUNT: usize = Stage::ApplyVectorMask as usize + 1;


pub trait Context: std::fmt::Debug {}


#[derive(Copy, Clone, Debug)]
pub struct MemoryCtx {
    pub pixels: *mut c_void,
    pub stride: u32, // can be zero
}

impl MemoryCtx {
    #[inline(always)]
    pub unsafe fn ptr_at_xy<T>(&self, dx: usize, dy: usize) -> *mut T {
        self.pixels.cast::<T>().add(self.stride as usize * dy + dx)
    }
}

impl Context for MemoryCtx {}


#[derive(Copy, Clone, Debug)]
pub struct GatherCtx {
    pub pixels: *mut c_void,
    pub stride: LengthU32,
    pub width: LengthU32,
    pub height: LengthU32,
}

impl Context for GatherCtx {}


#[derive(Copy, Clone, Debug)]
pub struct SamplerCtx {
    pub gather: GatherCtx,
    pub inv_width: f32,
    pub inv_height: f32,
}

impl Context for SamplerCtx {}


#[derive(Copy, Clone, Debug)]
pub struct UniformColorCtx {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub rgba: [u16; 4], // [0,255] in a 16-bit lane.
}

impl Context for UniformColorCtx {}


// A gradient color is an unpremultiplied RGBA not in a 0..1 range.
// It basically can have any float value.
#[derive(Copy, Clone, Default, Debug)]
pub struct GradientColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl GradientColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        GradientColor { r, g, b, a }
    }
}

impl From<Color> for GradientColor {
    fn from(c: Color) -> Self {
        GradientColor {
            r: c.red(),
            g: c.green(),
            b: c.blue(),
            a: c.alpha(),
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub struct EvenlySpaced2StopGradientCtx {
    pub factor: GradientColor,
    pub bias: GradientColor,
}

impl Context for EvenlySpaced2StopGradientCtx {}


#[derive(Clone, Default, Debug)]
pub struct GradientCtx {
    /// This value stores the actual colors count.
    /// `factors` and `biases` must store at least 16 values,
    /// since this is the length of a lowp pipeline stage.
    /// So any any value past `len` is just zeros.
    pub len: usize,
    pub factors: Vec<GradientColor>,
    pub biases: Vec<GradientColor>,
    pub t_values: Vec<NormalizedF32>,
}

impl GradientCtx {
    pub fn push_const_color(&mut self, color: GradientColor) {
        self.factors.push(GradientColor::new(0.0, 0.0, 0.0, 0.0));
        self.biases.push(color);
    }
}

impl Context for GradientCtx {}


#[derive(Copy, Clone, Debug)]
pub struct TwoPointConicalGradientCtx {
    // This context is used only in highp, where we use Tx4.
    pub mask: [u32; 4],
    pub p0: f32,
}

impl Context for TwoPointConicalGradientCtx {}


#[derive(Copy, Clone, Debug)]
pub struct TileCtx {
    pub scale: f32,
    pub inv_scale: f32, // cache of 1/scale
}

impl Context for TileCtx {}


impl Context for Transform {}


#[derive(Debug)]
pub struct ContextStorage {
    // TODO: stack array + fallback
    // TODO: find a better way
    // We cannot use just `c_void` here, like Skia,
    // because it will work only for POD types.
    items: Vec<*mut dyn Context>,
}

impl ContextStorage {
    pub fn new() -> Self {
        ContextStorage {
            items: Vec::new(),
        }
    }

    pub fn create_uniform_color_context(&mut self, c: PremultipliedColor) -> *const c_void {
        let r = c.red();
        let g = c.green();
        let b = c.blue();
        let a = c.alpha();
        let rgba = [
            (r * 255.0 + 0.5) as u16,
            (g * 255.0 + 0.5) as u16,
            (b * 255.0 + 0.5) as u16,
            (a * 255.0 + 0.5) as u16,
        ];

        let ctx = UniformColorCtx {
            r, g, b, a,
            rgba,
        };

        self.push_context(ctx)
    }

    pub fn push_context<T: Context + 'static>(&mut self, t: T) -> *const c_void {
        let ptr = Box::into_raw(Box::new(t));
        self.items.push(ptr);
        ptr as *const c_void
    }
}

impl Drop for ContextStorage {
    fn drop(&mut self) {
        for item in &self.items {
            unsafe { Box::from_raw(*item) };
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct StageList {
    stage: Stage,
    ctx: *const c_void,
}

pub struct RasterPipelineBuilder {
    // TODO: stack array + fallback
    stages: Vec<StageList>,
    slots_needed: usize,
    force_hq_pipeline: bool,
}

impl RasterPipelineBuilder {
    pub fn new() -> Self {
        RasterPipelineBuilder {
            stages: Vec::with_capacity(32),
            slots_needed: 1, // We always need one extra slot for just_return().
            force_hq_pipeline: false,
        }
    }

    pub fn is_force_hq_pipeline(&self) -> bool {
        self.force_hq_pipeline
    }

    pub fn set_force_hq_pipeline(&mut self, hq: bool) {
        self.force_hq_pipeline = hq;
    }

    pub fn len(&self) -> usize {
        self.stages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, stage: Stage) {
        self.unchecked_push(stage, std::ptr::null_mut());
    }

    pub fn push_with_context(&mut self, stage: Stage, ctx: *const c_void) {
        self.unchecked_push(stage, ctx);
    }

    pub fn push_transform(&mut self, ts: Transform, ctx_storage: &mut ContextStorage) {
        if ts.is_identity() {
            return;
        }

        let ctx = ctx_storage.push_context(ts);

        if ts.is_translate() {
            self.push_with_context(Stage::TransformTranslate, ctx);
        } else if ts.is_scale_translate() {
            self.push_with_context(Stage::TransformScaleTranslate, ctx);
        } else {
            self.push_with_context(Stage::Transform2X3, ctx);
        }
    }

    fn unchecked_push(&mut self, stage: Stage, ctx: *const c_void) {
        self.stages.push(StageList {
            stage,
            ctx,
        });

        self.slots_needed += if ctx.is_null() { 1 } else { 2 };
    }

    pub fn extend(&mut self, other: &Self) {
        if other.is_empty() {
            return;
        }

        self.stages.extend_from_slice(&other.stages);
        self.slots_needed += other.slots_needed - 1; // Don't double count just_return().
    }

    pub fn compile(&self) -> RasterPipeline {
        if self.stages.is_empty() {
            return RasterPipeline {
                program: Vec::new(),
                tail_program: Vec::new(),
                is_highp: false,
            };
        }

        let mut program: Vec<*const c_void> = Vec::with_capacity(self.slots_needed);

        let mut is_highp = self.force_hq_pipeline;
        if !self.force_hq_pipeline {
            for stage in &self.stages {
                let stage_fn = lowp::STAGES[stage.stage as usize];
                if !lowp::fn_ptr_eq(stage_fn, lowp::null_fn) {
                    program.push(lowp::fn_ptr(stage_fn));

                    if !stage.ctx.is_null() {
                        program.push(stage.ctx);
                    }
                } else {
                    is_highp = true;
                    break;
                }
            }
        }

        if is_highp {
            program.clear();

            for stage in &self.stages {
                let stage_fn = highp::STAGES[stage.stage as usize];
                program.push(highp::fn_ptr(stage_fn));

                if !stage.ctx.is_null() {
                    program.push(stage.ctx);
                }
            }

            program.push(highp::just_return as *const () as *const c_void);
        } else {
            program.push(lowp::just_return as *const () as *const c_void);
        }

        // I wasn't able to reproduce Skia's load_8888_/store_8888_ performance.
        // Skia uses fallthrough switch, which is probably the reason.
        // In Rust, any branching in load/store code drastically affects the performance.
        // So instead, we're using two "programs": one for "full stages" and one for "tail stages".
        // While the only difference is the load/store methods.
        let mut tail_program = program.clone();
        if is_highp {
            if let Some(idx) = tail_program.iter().position(|fun| *fun == highp::fn_ptr(highp::load_dst)) {
                tail_program[idx] = highp::fn_ptr(highp::load_dst_tail);
            }

            if let Some(idx) = tail_program.iter().position(|fun| *fun == highp::fn_ptr(highp::store)) {
                tail_program[idx] = highp::fn_ptr(highp::store_tail);
            }

            // SourceOverRgba calls load/store manually, without the pipeline,
            // therefore we have to switch it too.
            if let Some(idx) = tail_program.iter().position(|fun| *fun == highp::fn_ptr(highp::source_over_rgba)) {
                tail_program[idx] = highp::fn_ptr(highp::source_over_rgba_tail);
            }
        } else {
            if let Some(idx) = tail_program.iter().position(|fun| *fun == lowp::fn_ptr(lowp::load_dst)) {
                tail_program[idx] = lowp::fn_ptr(lowp::load_dst_tail);
            }

            if let Some(idx) = tail_program.iter().position(|fun| *fun == lowp::fn_ptr(lowp::store)) {
                tail_program[idx] = lowp::fn_ptr(lowp::store_tail);
            }

            // SourceOverRgba calls load/store manually, without the pipeline,
            // therefore we have to switch it too.
            if let Some(idx) = tail_program.iter().position(|fun| *fun == lowp::fn_ptr(lowp::source_over_rgba)) {
                tail_program[idx] = lowp::fn_ptr(lowp::source_over_rgba_tail);
            }
        }

        RasterPipeline {
            program,
            tail_program,
            is_highp,
        }
    }
}

pub struct RasterPipeline {
    // TODO: stack array + fallback
    program: Vec<*const c_void>,
    tail_program: Vec<*const c_void>,
    is_highp: bool,
}

impl RasterPipeline {
    pub fn run(&self, rect: &ScreenIntRect) {
        // Pipeline can be empty.
        if self.program.is_empty() {
            return;
        }

        if self.is_highp {
            highp::start(self.program.as_ptr(), self.tail_program.as_ptr(), rect);
        } else {
            lowp::start(self.program.as_ptr(), self.tail_program.as_ptr(), rect);
        }
    }
}


#[cfg(test)]
mod blend_tests {
    // Test blending modes.
    //
    // Skia has two kinds of a raster pipeline: high and low precision.
    // "High" uses f32 and "low" uses u16.
    // And for basic operations we don't need f32 and u16 simply faster.
    // But those modes are not identical. They can produce slightly different results
    // due rounding.

    use super::*;
    use crate::{Pixmap, Painter, Color, PremultipliedColorU8, BlendMode};

    macro_rules! test_blend {
        ($name:ident, $mode:expr, $is_highp:expr, $r:expr, $g:expr, $b:expr, $a:expr) => {
            #[test]
            fn $name() {
                let mut pixmap = Pixmap::new(1, 1).unwrap();
                pixmap.fill(Color::from_rgba8(50, 127, 150, 200));

                let img_ctx = MemoryCtx {
                    pixels: pixmap.data().as_ptr() as _,
                    stride: pixmap.size().width(),
                };
                let img_ctx = &img_ctx as *const _ as *const c_void;

                let mut ctx_storage = ContextStorage::new();
                let color_ctx = ctx_storage.create_uniform_color_context(
                    Color::from_rgba8(220, 140, 75, 180).premultiply(),
                );

                let mut p = RasterPipelineBuilder::new();
                p.set_force_hq_pipeline($is_highp);
                p.push_with_context(Stage::UniformColor, color_ctx);
                p.push_with_context(Stage::LoadDestination, img_ctx);
                p.push($mode.to_stage().unwrap());
                p.push_with_context(Stage::Store, img_ctx);
                let p = p.compile();
                p.run(&pixmap.size().to_screen_int_rect(0, 0));

                assert_eq!(p.is_highp, $is_highp);

                assert_eq!(
                    pixmap.pixel(0, 0).unwrap(),
                    PremultipliedColorU8::from_rgba($r, $g, $b, $a).unwrap()
                );
            }
        };
    }

    macro_rules! test_blend_lowp {
        ($name:ident, $mode:expr, $r:expr, $g:expr, $b:expr, $a:expr) => (
            test_blend!{$name, $mode, false, $r, $g, $b, $a}
        )
    }

    macro_rules! test_blend_highp {
        ($name:ident, $mode:expr, $r:expr, $g:expr, $b:expr, $a:expr) => (
            test_blend!{$name, $mode, true, $r, $g, $b, $a}
        )
    }

    test_blend_lowp!(clear_lowp,              BlendMode::Clear,                 0,   0,   0,   0);
    // Source is a no-op
    test_blend_lowp!(destination_lowp,        BlendMode::Destination,          39, 100, 118, 200);
    test_blend_lowp!(source_over_lowp,        BlendMode::SourceOver,          167, 129,  88, 239);
    test_blend_lowp!(destination_over_lowp,   BlendMode::DestinationOver,      73, 122, 130, 239);
    test_blend_lowp!(source_in_lowp,          BlendMode::SourceIn,            122,  78,  42, 141);
    test_blend_lowp!(destination_in_lowp,     BlendMode::DestinationIn,        28,  71,  83, 141);
    test_blend_lowp!(source_out_lowp,         BlendMode::SourceOut,            34,  22,  12,  39);
    test_blend_lowp!(destination_out_lowp,    BlendMode::DestinationOut,       12,  30,  35,  59);
    test_blend_lowp!(source_atop_lowp,        BlendMode::SourceAtop,          133, 107,  76, 200);
    test_blend_lowp!(destination_atop_lowp,   BlendMode::DestinationAtop,      61,  92,  95, 180);
    test_blend_lowp!(xor_lowp,                BlendMode::Xor,                  45,  51,  46,  98);
    test_blend_lowp!(plus_lowp,               BlendMode::Plus,                194, 199, 171, 255);
    test_blend_lowp!(modulate_lowp,           BlendMode::Modulate,             24,  39,  25, 141);
    test_blend_lowp!(screen_lowp,             BlendMode::Screen,              170, 160, 146, 239);
    test_blend_lowp!(overlay_lowp,            BlendMode::Overlay,              92, 128, 106, 239);
    test_blend_lowp!(darken_lowp,             BlendMode::Darken,               72, 121,  88, 239);
    test_blend_lowp!(lighten_lowp,            BlendMode::Lighten,             166, 128, 129, 239);
    // ColorDodge in not available for lowp.
    // ColorBurn in not available for lowp.
    test_blend_lowp!(hard_light_lowp,         BlendMode::HardLight,           154, 128,  95, 239);
    // SoftLight in not available for lowp.
    test_blend_lowp!(difference_lowp,         BlendMode::Difference,          138,  57,  87, 239);
    test_blend_lowp!(exclusion_lowp,          BlendMode::Exclusion,           146, 121, 121, 239);
    test_blend_lowp!(multiply_lowp,           BlendMode::Multiply,             69,  90,  71, 238);
    // Hue in not available for lowp.
    // Saturation in not available for lowp.
    // Color in not available for lowp.
    // Luminosity in not available for lowp.

    test_blend_highp!(clear_highp,            BlendMode::Clear,                 0,   0,   0,   0);
    // Source is a no-op
    test_blend_highp!(destination_highp,      BlendMode::Destination,          39, 100, 118, 200);
    test_blend_highp!(source_over_highp,      BlendMode::SourceOver,          167, 128,  88, 239);
    test_blend_highp!(destination_over_highp, BlendMode::DestinationOver,      72, 121, 129, 239);
    test_blend_highp!(source_in_highp,        BlendMode::SourceIn,            122,  78,  42, 141);
    test_blend_highp!(destination_in_highp,   BlendMode::DestinationIn,        28,  71,  83, 141);
    test_blend_highp!(source_out_highp,       BlendMode::SourceOut,            33,  21,  11,  39);
    test_blend_highp!(destination_out_highp,  BlendMode::DestinationOut,       11,  29,  35,  59);
    test_blend_highp!(source_atop_highp,      BlendMode::SourceAtop,          133, 107,  76, 200);
    test_blend_highp!(destination_atop_highp, BlendMode::DestinationAtop,      61,  92,  95, 180);
    test_blend_highp!(xor_highp,              BlendMode::Xor,                  45,  51,  46,  98);
    test_blend_highp!(plus_highp,             BlendMode::Plus,                194, 199, 171, 255);
    test_blend_highp!(modulate_highp,         BlendMode::Modulate,             24,  39,  24, 141);
    test_blend_highp!(screen_highp,           BlendMode::Screen,              171, 160, 146, 239);
    test_blend_highp!(overlay_highp,          BlendMode::Overlay,              92, 128, 106, 239);
    test_blend_highp!(darken_highp,           BlendMode::Darken,               72, 121,  88, 239);
    test_blend_highp!(lighten_highp,          BlendMode::Lighten,             167, 128, 129, 239);
    test_blend_highp!(color_dodge_highp,      BlendMode::ColorDodge,          186, 192, 164, 239);
    test_blend_highp!(color_burn_highp,       BlendMode::ColorBurn,            54,  63,  46, 239);
    test_blend_highp!(hard_light_highp,       BlendMode::HardLight,           155, 128,  95, 239);
    test_blend_highp!(soft_light_highp,       BlendMode::SoftLight,            98, 124, 115, 239);
    test_blend_highp!(difference_highp,       BlendMode::Difference,          139,  58,  88, 239);
    test_blend_highp!(exclusion_highp,        BlendMode::Exclusion,           147, 121, 122, 239);
    test_blend_highp!(multiply_highp,         BlendMode::Multiply,             69,  89,  71, 239);
    test_blend_highp!(hue_highp,              BlendMode::Hue,                 128, 103,  74, 239);
    test_blend_highp!(saturation_highp,       BlendMode::Saturation,           59, 126, 140, 239);
    test_blend_highp!(color_highp,            BlendMode::Color,               139, 100,  60, 239);
    test_blend_highp!(luminosity_highp,       BlendMode::Luminosity,          100, 149, 157, 239);
}
