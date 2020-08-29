// Copyright 2016 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::ffi::c_void;

use crate::ScreenIntRect;

use crate::color::PremultipliedColor;

pub mod ffi {
    #![allow(non_camel_case_types)]

    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct skia_pipe_stage_list {
        _unused: [u8; 0],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct sk_raster_pipeline_memory_ctx {
        pub pixels: *mut c_void,
        pub stride: i32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct sk_raster_pipeline_uniform_color_ctx {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
        pub rgba: [u16; 4], // [0,255] in a 16-bit lane.
    }

    extern "C" {
        pub fn skia_pipe_raster_build_pipeline(
            stages: *mut skia_pipe_stage_list,
            ip: *mut *mut c_void,
        ) -> bool;

        pub fn skia_pipe_raster_run_pipeline(
            program: *const *mut c_void,
            is_highp: bool,
            x: u32,
            y: u32,
            w: u32,
            h: u32,
        );
    }
}


#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum Stage {
    MoveSourceToDestination = 0,
    MoveDestinationToSource,
    Clamp0,
    Clamp1,
    ClampA,
    ClampGamut,
    Unpremultiply,
    Premultiply,
    PremultiplyDestination,
    BlackColor,
    WhiteColor,
    UniformColor,
    UnboundedUniformColor,
    UniformColorDestination,
    SeedShader,
    Dither,
    Load,
    LoadDestination,
    Store,
    Gather,
    BilerpClamp,
    BicubicClamp,
    ScaleU8,
    Scale1Float,
    ScaleNative,
    LerpU8,
    Lerp1Float,
    LerpNative,
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
    MatrixTranslate,
    MatrixScaleTranslate,
    Matrix2X3,
    Matrix3X3,
    Matrix3X4,
    Matrix4X5,
    Matrix4X3,
    MirrorX,
    RepeatX,
    MirrorY,
    RepeatY,
    NegateX,
    Bilinear,
    Bicubic,
    BilinearNX,
    BilinearPX,
    BilinearNY,
    BilinearPY,
    BicubicN3X,
    BicubicN1X,
    BicubicP1X,
    BicubicP3X,
    BicubicN3Y,
    BicubicN1Y,
    BicubicP1Y,
    BicubicP3Y,
    SaveXY,
    Accumulate,
    ClampX1,
    MirrorX1,
    RepeatX1,
    EvenlySpacedGradient,
    Gradient,
    EvenlySpaced2StopGradient,
    XyToUnitAngle,
    XYToRadius,
    XYTo2PtConicalStrip,
    XYTo2PtConicalFocalOnCircle,
    XYTo2PtConicalWellBehaved,
    XYTo2PtConicalSmaller,
    XYTo2PtConicalGreater,
    Alter2PtConicalCompensateFocal,
    Alter2PtConicalUnswap,
    Mask2PtConicalNan,
    Mask2PtConicalDegenerates,
    ApplyVectorMask,
}


pub struct ContextStorage {
    // TODO: stack array + fallback
    // TODO: find a better way
    items: Vec<*const c_void>,
}

impl ContextStorage {
    #[inline]
    pub fn new() -> Self {
        ContextStorage {
            items: Vec::new(),
        }
    }

    #[inline]
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

        let ctx = ffi::sk_raster_pipeline_uniform_color_ctx {
            r, g, b, a,
            rgba,
        };

        self.push_context(ctx)
    }

    #[inline]
    pub fn push_context<T>(&mut self, t: T) -> *const c_void {
        let ptr = Box::into_raw(Box::new(t)) as *const c_void;
        self.items.push(ptr);
        ptr
    }
}

impl Drop for ContextStorage {
    #[inline]
    fn drop(&mut self) {
        for item in &self.items {
            unsafe { Box::from_raw(*item as *mut c_void) };
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct StageList {
    prev: *mut StageList,
    stage: i32,
    ctx: *const c_void,
}

pub struct RasterPipelineBuilder {
    // TODO: stack array + fallback
    stages: Vec<StageList>,
    slots_needed: usize,
}

impl RasterPipelineBuilder {
    #[inline]
    pub fn new() -> Self {
        RasterPipelineBuilder {
            stages: Vec::with_capacity(32),
            slots_needed: 1, // We always need one extra slot for just_return().
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn push(&mut self, stage: Stage) {
        self.unchecked_push(stage as i32, std::ptr::null_mut());
    }

    #[inline]
    pub fn push_with_context(&mut self, stage: Stage, ctx: *const c_void) {
        self.unchecked_push(stage as i32, ctx);
    }

    #[inline]
    fn unchecked_push(&mut self, stage: i32, ctx: *const c_void) {
        let prev = if self.stages.is_empty() {
            std::ptr::null_mut()
        } else {
            unsafe { self.stages.as_mut_ptr().add(self.stages.len() - 1) }
        };

        self.stages.push(StageList {
            prev,
            stage,
            ctx,
        });

        self.slots_needed += if ctx.is_null() { 1 } else { 2 };
    }

    #[inline]
    pub fn extend(&mut self, other: &Self) {
        if other.is_empty() {
            return;
        }

        self.stages.extend_from_slice(&other.stages);
        self.slots_needed += other.slots_needed - 1; // Don't double count just_returns().
    }

    #[inline]
    pub fn compile(&self) -> RasterPipeline {
        if self.stages.is_empty() {
            return RasterPipeline {
                program: Vec::new(),
                is_highp: false,
            };
        }

        let mut program: Vec<*mut c_void> = vec![std::ptr::null_mut(); self.slots_needed];

        let is_highp = unsafe {
            // Skia builds a pipeline from the end.
            let last_stage = self.stages.as_ptr().add(self.stages.len() - 1);
            ffi::skia_pipe_raster_build_pipeline(
                last_stage as _,
                program.as_mut_ptr().add(self.slots_needed)
            )
        };

        RasterPipeline {
            program,
            is_highp,
        }
    }
}

pub struct RasterPipeline {
    // TODO: stack array + fallback
    program: Vec<*mut c_void>,
    is_highp: bool,
}

impl RasterPipeline {
    #[inline]
    pub fn run(&self, rect: ScreenIntRect) {
        // Pipeline can be empty.
        if self.program.is_empty() {
            return;
        }

        unsafe {
            ffi::skia_pipe_raster_run_pipeline(
                self.program.as_ptr(),
                self.is_highp,
                rect.x(),
                rect.y(),
                rect.width(),
                rect.height(),
            );
        }
    }
}
