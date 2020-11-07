// Copyright 2016 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/*!
A raster pipeline implementation.

Despite having a lot of changes compared to `SkRasterPipeline`,
the core principles are the same:

1. A pipeline consists of stages.
1. Each stage can have an optional context.
1. Each stage has a high precision implementation. See `highp.rs`.
1. Some stages have a low precision (lowp) implementation. See `lowp.rs`.
1. Each stage calls the "next" stage after its done.
1. During pipeline "compilation", if **all** stages have a lowp implementation,
   the lowp pipeline will be used. Otherwise, the highp variant will be used.
1. The pipeline "compilation" produces a list of `void` pointer pairs (aka program),
   were the first pointer is a stage function pointer and the second one is an optional context.
   The last pointer is a pointer to the "return" function,
   which simply stops the execution of the pipeline.
   So the program can look like this: `[fn, ctx, fn, fn, fn, ctx, ret_fn]`.

A simple illustration:

```ignore
type StageFn = fn(program: *const *const c_void);

fn start(program: *const *const c_void) {
    let next: StageFn = unsafe { *program.cast() };
    next(program);
}

fn stage1(program: *const *const c_void) {
    // This stage has a context.
    let ctx: &SomeT = unsafe { &*(*program.add(1)).cast() };

    // Do stuff...

    unsafe {
        let next: StageFn = *program.add(2).cast(); // advance by 2, because of ctx
        next(program.add(2));
    }
}

fn stage2(program: *const *const c_void) {
    // This stage has no context.

    // Do stuff...

    unsafe {
        let next: StageFn = *program.add(1).cast(); // advance by 1, because no ctx
        next(program.add(1));
    }
}

fn just_return(_: *const *const c_void) {
    // stops the execution
}

// Were the `program` can look like: `[*stage1, *SomeT, *stage2, *just_return]`
```

This implementation is a bit tricky, but it gives the maximum performance.
A simple and straightforward implementation using traits and loops, like:

```ignore
trait StageTrait {
    fn apply(&mut self, pixels: &mut [Pixel]);
}

let stages: Vec<&mut dyn StageTrait>;
for stage in stages {
    stage.apply(pixels);
}
```

will be at least 20-30% slower. Not really sure why.

The main problem with function pointers approach, is that there is no way
we can have a list of random data and safely cast it without any overhead.
Therefore we have to really on unsafe. A stage function simply "knows" that
there are enough pointers left in the `program`.

Moreover, since this module is all about performance, any kind of branching is
strictly forbidden. All stage functions must not use `if`, `match` or loops.
There are still some exceptions, which are basically an imperfect implementations
and should be optimized out in the future.

Despite all the above, we still have a fully checked pixels access.
Only program pointers casting (to fn and ctx) and data types mutations are unsafe.
*/

use std::ffi::c_void;
use std::rc::Rc;

use arrayvec::ArrayVec;

use crate::{LengthU32, Color, SpreadMode, PremultipliedColor, PremultipliedColorU8};

pub use blitter::RasterPipelineBlitter;

use crate::floating_point::NormalizedF32;
use crate::screen_int_rect::ScreenIntRect;
use crate::transform::TransformUnchecked;
use crate::wide::u32x8;

mod blitter;
mod lowp;
mod highp;


const MAX_STAGES: usize = 32; // More than enough.


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
    MaskU8,
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
    Transform,
    ReflectX,
    ReflectY,
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


impl Context for f32 {}


#[derive(Debug)]
pub struct PixelsCtx<'a> {
    pub pixels: &'a mut [PremultipliedColorU8],
    pub stride: LengthU32,
}

impl PixelsCtx<'_> {
    #[inline(always)]
    fn offset(&self, dx: usize, dy: usize) -> usize {
        self.stride.get() as usize * dy + dx
    }

    #[inline(always)]
    pub fn slice_at_xy(&mut self, dx: usize, dy: usize) -> &mut [PremultipliedColorU8] {
        let offset = self.offset(dx, dy);
        &mut self.pixels[offset..]
    }

    #[inline(always)]
    pub fn slice4_at_xy(&mut self, dx: usize, dy: usize) -> &mut [PremultipliedColorU8; highp::STAGE_WIDTH] {
        arrayref::array_mut_ref!(self.pixels, self.offset(dx, dy), highp::STAGE_WIDTH)
    }

    #[inline(always)]
    pub fn slice16_at_xy(&mut self, dx: usize, dy: usize) -> &mut [PremultipliedColorU8; lowp::STAGE_WIDTH] {
        arrayref::array_mut_ref!(self.pixels, self.offset(dx, dy), lowp::STAGE_WIDTH)
    }
}

impl Context for PixelsCtx<'_> {}


#[derive(Default, Debug)]
pub struct MaskCtx {
    pub pixels: [u8; 2],
    pub stride: u32, // can be zero
    pub shift: usize, // mask offset/position in pixmap coordinates
}

impl MaskCtx {
    #[inline(always)]
    pub fn copy_at_xy(&self, dx: usize, dy: usize, tail: usize) -> [u8; 2] {
        let offset = (self.stride as usize * dy + dx) - self.shift;
        // We have only 3 variants, so unroll them.
        match (offset, tail) {
            (0, 1) => [self.pixels[0], 0],
            (0, 2) => [self.pixels[0], self.pixels[1]],
            (1, 1) => [self.pixels[1], 0],
            _ => [0, 0] // unreachable
        }
    }
}

impl Context for MaskCtx {}


// TODO: merge with MaskCtx
#[derive(Debug)]
pub struct ClipMaskCtx<'a> {
    pub data: &'a [u8],
    pub stride: LengthU32,
}

impl ClipMaskCtx<'_> {
    #[inline(always)]
    fn offset(&self, dx: usize, dy: usize) -> usize {
        self.stride.get() as usize * dy + dx
    }
}

impl Context for ClipMaskCtx<'_> {}


#[derive(Copy, Clone, Debug)]
pub struct GatherCtx {
    // We have to use a pointer to bypass lifetime restrictions.
    // The access is still bound checked.
    pub pixels: *const PremultipliedColorU8,
    pub pixels_len: usize,
    pub width: LengthU32,
    pub height: LengthU32,
}

impl GatherCtx {
    #[inline(always)]
    pub fn gather(&self, index: u32x8) -> [PremultipliedColorU8; highp::STAGE_WIDTH] {
        // TODO: remove unsafe
        let pixels = unsafe { std::slice::from_raw_parts(self.pixels, self.pixels_len) };
        let index: [u32; 8] = index.into();
        [
            pixels[index[0] as usize],
            pixels[index[1] as usize],
            pixels[index[2] as usize],
            pixels[index[3] as usize],
            pixels[index[4] as usize],
            pixels[index[5] as usize],
            pixels[index[6] as usize],
            pixels[index[7] as usize],
        ]
    }
}

impl Context for GatherCtx {}


#[derive(Copy, Clone, Debug)]
pub struct SamplerCtx {
    pub gather: GatherCtx,
    pub spread_mode: SpreadMode,
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
    pub mask: u32x8,
    pub p0: f32,
}

impl Context for TwoPointConicalGradientCtx {}


#[derive(Copy, Clone, Debug)]
pub struct TileCtx {
    pub scale: f32,
    pub inv_scale: f32, // cache of 1/scale
}

impl Context for TileCtx {}


impl Context for TransformUnchecked {}


pub struct ContextStorage {
    // We have to use Rc because Box doesn't provide as_pts method.
    items: ArrayVec<[Rc<dyn Context>; MAX_STAGES]>,
}

impl ContextStorage {
    pub fn new() -> Self {
        ContextStorage {
            items: ArrayVec::new(),
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
        let rc = Rc::new(t);
        let ptr = Rc::as_ptr(&rc) as *const c_void;
        self.items.push(rc);
        ptr
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct StageList {
    stage: Stage,
    ctx: *const c_void,
}

pub struct RasterPipelineBuilder {
    stages: ArrayVec<[StageList; MAX_STAGES]>,
    slots_needed: usize,
    force_hq_pipeline: bool,
}

impl RasterPipelineBuilder {
    pub fn new() -> Self {
        RasterPipelineBuilder {
            stages: ArrayVec::new(),
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

    pub fn push_transform(&mut self, ts: TransformUnchecked, ctx_storage: &mut ContextStorage) {
        if !ts.is_identity() {
            let ctx = ctx_storage.push_context(ts);
            self.push_with_context(Stage::Transform, ctx);
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
        if !other.is_empty() {
            self.stages.try_extend_from_slice(&other.stages).unwrap();
            self.slots_needed += other.slots_needed - 1; // Don't double count just_return().
        }
    }

    pub fn compile(&self) -> RasterPipeline {
        if self.stages.is_empty() {
            return RasterPipeline {
                program: ArrayVec::new(),
                tail_program: ArrayVec::new(),
                is_highp: false,
            };
        }

        let mut program = ArrayVec::new();

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

        // TODO: trim to 100

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
    program: ArrayVec<[*const c_void; MAX_STAGES * 2]>, // 2x because we have fn + ?ctx.
    tail_program: ArrayVec<[*const c_void; MAX_STAGES * 2]>,
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


// We cannot guarantee that `unsafe` in this trait is actually safe.
// Our only choice is to run tests under address sanitizer, which we do.
trait BasePipeline {
    fn program(&self) -> *const *const c_void;
    fn set_program(&mut self, p: *const *const c_void);

    #[inline(always)]
    fn next_stage(&mut self, offset: usize) {
        unsafe {
            self.set_program(self.program().add(offset));

            let next: fn(&mut Self) = *self.program().cast();
            next(self);
        }
    }

    #[inline(always)]
    fn stage_ctx<T>(&self) -> &'static T {
        unsafe { &*(*self.program().add(1)).cast() }
    }

    #[inline(always)]
    fn stage_ctx_mut<T>(&mut self) -> &'static mut T {
        // We have to cast `*const` to `*mut` first.
        // TODO: this is logically incorrect since we're changing the mutability.
        unsafe { &mut *(*self.program().add(1) as *mut c_void).cast() }
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
    use crate::{Canvas, Color, PremultipliedColorU8, BlendMode};

    macro_rules! test_blend {
        ($name:ident, $mode:expr, $is_highp:expr, $r:expr, $g:expr, $b:expr, $a:expr) => {
            #[test]
            fn $name() {
                let mut canvas = Canvas::new(1, 1).unwrap();
                canvas.fill_canvas(Color::from_rgba8(50, 127, 150, 200));

                let img_ctx = PixelsCtx {
                    stride: canvas.pixmap.size().width_safe(),
                    pixels: canvas.pixmap.pixels_mut(),
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
                p.run(&canvas.pixmap.size().to_screen_int_rect(0, 0));

                assert_eq!(p.is_highp, $is_highp);

                assert_eq!(
                    canvas.pixmap.pixel(0, 0).unwrap(),
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
