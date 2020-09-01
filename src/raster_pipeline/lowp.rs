// Copyright 2018 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/*!
A low precision raster pipeline implementation.

A lowp pipeline uses u16 instead of f32 for math.
Because of that, it doesn't implement stages that require high precision.
The pipeline compiler will automatically decide which one to use.

Skia uses u16x8 (128bit) types for a generic CPU and u16x16 (256bit) for modern x86 CPUs.
But instead of explicit SIMD instructions, it mainly relies on clang's vector extensions.
And since they are unavailable in Rust, we have to do everything manually.

According to benchmarks, a SIMD-accelerated u16x8 in Rust is almost 2x slower than in Skia.
Not sure why. For example, there are no div instruction for u16x8, so we have to use
a basic scalar version. Which means unnecessary load/store. No idea what clang does in this case.
Surprisingly, a SIMD-accelerated u16x8 is even slower than a scalar one. Not sure why.

Unlike Skia, we are using u16x16 by default and relying on rustc/llvm auto vectorization instead.
When targeting a generic CPU, we're just 5-10% slower than Skia. While u16x8 is 30-40% slower.
And while `-C target-cpu=haswell` boosts our performance by around 25%,
we are still 40-60% behind Skia built for haswell.
*/

use std::ffi::c_void;

use crate::{ScreenIntRect, PremultipliedColorU8};

use crate::raster_pipeline::{self, STAGES_COUNT};
use crate::wide::U16x16;

const STAGE_WIDTH: usize = 16;

type StageFn = unsafe fn(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
);

// Must be in the same order as raster_pipeline::Stage
pub const STAGES: &[StageFn; STAGES_COUNT] = &[
    move_source_to_destination,
    move_destination_to_source,
    null_fn, // Clamp0,
    null_fn, // Clamp1,
    null_fn, // ClampA,
    null_fn, // ClampGamut,
    null_fn, // Unpremultiply,
    null_fn, // Premultiply,
    null_fn, // PremultiplyDestination,
    null_fn, // BlackColor,
    null_fn, // WhiteColor,
    uniform_color,
    null_fn, // UnboundedUniformColor,
    null_fn, // UniformColorDestination,
    null_fn, // SeedShader,
    null_fn, // Dither,
    null_fn, // Load,
    load_dst,
    store,
    null_fn, // Gather,
    null_fn, // BilerpClamp,
    null_fn, // BicubicClamp,
    null_fn, // ScaleU8,
    scale_1_float, // Scale1Float,
    null_fn, // ScaleNative,
    null_fn, // LerpU8,
    lerp_1_float, // Lerp1Float,
    null_fn, // LerpNative,
    destination_atop,
    destination_in,
    destination_out,
    destination_over,
    source_atop,
    source_in,
    source_out,
    source_over,
    clear,
    modulate,
    multiply,
    plus,
    screen,
    xor,
    null_fn, // ColorBurn
    null_fn, // ColorDodge
    darken,
    difference,
    exclusion,
    hard_light,
    lighten,
    overlay,
    null_fn, // SoftLight
    null_fn, // Hue
    null_fn, // Saturation
    null_fn, // Color
    null_fn, // Luminosity
    source_over_rgba,
    null_fn, // MatrixTranslate,
    null_fn, // MatrixScaleTranslate,
    null_fn, // Matrix2X3,
    null_fn, // MirrorX,
    null_fn, // RepeatX,
    null_fn, // MirrorY,
    null_fn, // RepeatY,
    null_fn, // NegateX,
    null_fn, // Bilinear,
    null_fn, // Bicubic,
    null_fn, // BilinearNX,
    null_fn, // BilinearPX,
    null_fn, // BilinearNY,
    null_fn, // BilinearPY,
    null_fn, // BicubicN3X,
    null_fn, // BicubicN1X,
    null_fn, // BicubicP1X,
    null_fn, // BicubicP3X,
    null_fn, // BicubicN3Y,
    null_fn, // BicubicN1Y,
    null_fn, // BicubicP1Y,
    null_fn, // BicubicP3Y,
    null_fn, // SaveXY,
    null_fn, // Accumulate,
    null_fn, // ClampX1,
    null_fn, // MirrorX1,
    null_fn, // RepeatX1,
    null_fn, // EvenlySpacedGradient,
    null_fn, // Gradient,
    null_fn, // EvenlySpaced2StopGradient,
    null_fn, // XyToUnitAngle,
    null_fn, // XYToRadius,
    null_fn, // XYTo2PtConicalStrip,
    null_fn, // XYTo2PtConicalFocalOnCircle,
    null_fn, // XYTo2PtConicalWellBehaved,
    null_fn, // XYTo2PtConicalSmaller,
    null_fn, // XYTo2PtConicalGreater,
    null_fn, // Alter2PtConicalCompensateFocal,
    null_fn, // Alter2PtConicalUnswap,
    null_fn, // Mask2PtConicalNan,
    null_fn, // Mask2PtConicalDegenerates,
    null_fn, // ApplyVectorMask,
];

#[inline]
pub fn fn_ptr(f: StageFn) -> *const c_void {
    f as *const () as *const c_void
}

#[inline]
pub fn fn_ptr_eq(f1: StageFn, f2: StageFn) -> bool {
    f1 as *const () == f2 as *const ()
}

#[inline(never)]
pub fn start(
    program: *const *const c_void,
    tail_program: *const *const c_void,
    rect: ScreenIntRect,
) {
    let mut  r = U16x16::default();
    let mut  g = U16x16::default();
    let mut  b = U16x16::default();
    let mut  a = U16x16::default();
    let mut dr = U16x16::default();
    let mut dg = U16x16::default();
    let mut db = U16x16::default();
    let mut da = U16x16::default();

    for y in rect.y()..rect.bottom() {
        let mut x = rect.x() as usize;
        let end = rect.right() as usize;

        while x + STAGE_WIDTH <= end {
            unsafe {
                let next: StageFn = *program.cast();
                next(
                    STAGE_WIDTH, program, x, y as usize,
                    &mut r, &mut g, &mut b, &mut a,
                    &mut dr, &mut dg, &mut db, &mut da,
                );
            }

            x += STAGE_WIDTH;
        }

        if x != end {
            unsafe {
                let next: StageFn = *tail_program.cast();
                next(
                    end - x, tail_program, x, y as usize,
                    &mut r, &mut g, &mut b, &mut a,
                    &mut dr, &mut dg, &mut db, &mut da,
                );
            }
        }
    }
}

unsafe fn move_source_to_destination(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    *dr = *r;
    *dg = *g;
    *db = *b;
    *da = *a;

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn move_destination_to_source(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    *r = *dr;
    *g = *dg;
    *b = *db;
    *a = *da;

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn uniform_color(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &raster_pipeline::UniformColorCtx = &*(*program.add(1)).cast();
    *r = U16x16::splat(ctx.rgba[0]);
    *g = U16x16::splat(ctx.rgba[1]);
    *b = U16x16::splat(ctx.rgba[2]);
    *a = U16x16::splat(ctx.rgba[3]);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn load_dst(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_(ptr, dr, dg, db, da);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn load_dst_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_tail_(tail, ptr, dr, dg, db, da);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn store(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    store_8888_(ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn store_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    store_8888_tail_(tail, ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn scale_1_float(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let c: f32 = *(*program.add(1)).cast();
    let c = from_float(c);
    *r = div255(*r * c);
    *g = div255(*g * c);
    *b = div255(*b * c);
    *a = div255(*a * c);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn lerp_1_float(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let c: f32 = *(*program.add(1)).cast();
    let c = from_float(c);
    *r = lerp(*dr, *r, c);
    *g = lerp(*dg, *g, c);
    *b = lerp(*db, *b, c);
    *a = lerp(*da, *a, c);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

macro_rules! blend_fn {
    ($name:ident, $f:expr) => {
        unsafe fn $name(
            tail: usize, program: *const *const c_void, dx: usize, dy: usize,
            r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
            dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
        ) {
            *r = $f(*r, *dr, *a, *da);
            *g = $f(*g, *dg, *a, *da);
            *b = $f(*b, *db, *a, *da);
            *a = $f(*a, *da, *a, *da);

            let next: StageFn = *program.add(1).cast();
            next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
        }
    };
}

blend_fn!(clear,            |_, _,  _,  _| U16x16::splat(0));
blend_fn!(source_atop,      |s, d, sa, da| div255(s * da + d * inv(sa)));
blend_fn!(destination_atop, |s, d, sa, da| div255(d * sa + s * inv(da)));
blend_fn!(source_in,        |s, _,  _, da| div255(s * da));
blend_fn!(destination_in,   |_, d, sa,  _| div255(d * sa));
blend_fn!(source_out,       |s, _,  _, da| div255(s * inv(da)));
blend_fn!(destination_out,  |_, d, sa,  _| div255(d * inv(sa)));
blend_fn!(source_over,      |s, d, sa,  _| s + div255(d * inv(sa)));
blend_fn!(destination_over, |s, d,  _, da| d + div255(s * inv(da)));
blend_fn!(modulate,         |s, d,  _,  _| div255(s * d));
blend_fn!(multiply,         |s, d, sa, da| div255(s * inv(da) + d * inv(sa) + s * d));
blend_fn!(screen,           |s, d,  _,  _| s + d - div255(s * d));
blend_fn!(xor,              |s, d, sa, da| div255(s * inv(da) + d * inv(sa)));

// Wants a type for some reason.
blend_fn!(plus, |s: U16x16, d, _, _| (s + d).min(&U16x16::splat(255)));


macro_rules! blend_fn2 {
    ($name:ident, $f:expr) => {
        unsafe fn $name(
            tail: usize, program: *const *const c_void, dx: usize, dy: usize,
            r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
            dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
        ) {
            // The same logic applied to color, and source_over for alpha.
            *r = $f(*r, *dr, *a, *da);
            *g = $f(*g, *dg, *a, *da);
            *b = $f(*b, *db, *a, *da);
            *a = *a + div255(*da * inv(*a));

            let next: StageFn = *program.add(1).cast();
            next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
        }
    };
}

blend_fn2!(darken,      |s: U16x16, d, sa, da| s + d - div255((s * da).max(&(d * sa))));
blend_fn2!(lighten,     |s: U16x16, d, sa, da| s + d - div255((s * da).min(&(d * sa))));
blend_fn2!(exclusion,   |s: U16x16, d,  _,  _| s + d - U16x16::splat(2) * div255(s * d));

blend_fn2!(difference,  |s: U16x16, d, sa, da|
    s + d - U16x16::splat(2) * div255((s * da).min(&(d * sa))));

blend_fn2!(hard_light, |s: U16x16, d: U16x16, sa, da| {
    div255(s * inv(da) + d * inv(sa)
        + (s+s).packed_le(sa).if_then_else(
            U16x16::splat(2) * s * d,
            sa * da - U16x16::splat(2) * (sa-s)*(da-d)
        )
    )
});

blend_fn2!(overlay, |s: U16x16, d: U16x16, sa, da| {
    div255(s * inv(da) + d * inv(sa)
        + (d+d).packed_le(da).if_then_else(
            U16x16::splat(2) * s * d,
            sa * da - U16x16::splat(2) * (sa-s)*(da-d)
        )
    )
});

pub unsafe fn source_over_rgba(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_(ptr, dr, dg, db, da);
    *r = *r + div255(*dr * inv(*a));
    *g = *g + div255(*dg * inv(*a));
    *b = *b + div255(*db * inv(*a));
    *a = *a + div255(*da * inv(*a));
    store_8888_(ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn source_over_rgba_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_tail_(tail, ptr, dr, dg, db, da);
    *r = *r + div255(*dr * inv(*a));
    *g = *g + div255(*dg * inv(*a));
    *b = *b + div255(*db * inv(*a));
    *a = *a + div255(*da * inv(*a));
    store_8888_tail_(tail, ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn just_return(
    _: usize, _: *const *const c_void, _: usize, _: usize,
    _: &mut U16x16, _: &mut U16x16, _: &mut U16x16, _: &mut U16x16,
    _: &mut U16x16, _: &mut U16x16, _: &mut U16x16, _: &mut U16x16,
) {
    // Ends the loop.
}

pub unsafe fn null_fn(
    _: usize, _: *const *const c_void, _: usize, _: usize,
    _: &mut U16x16, _: &mut U16x16, _: &mut U16x16, _: &mut U16x16,
    _: &mut U16x16, _: &mut U16x16, _: &mut U16x16, _: &mut U16x16,
) {
    // Just for unsupported functions in STAGES.
}

#[inline(always)]
unsafe fn load_8888_(
    ptr: *const PremultipliedColorU8,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
) {
    // Cast a data pointer to a fixed size array.
    let data = &*(ptr as *const [PremultipliedColorU8; STAGE_WIDTH]);
    load_8888__(data, r, g, b, a);
}

#[inline(always)]
unsafe fn load_8888_tail_(
    tail: usize, ptr: *const PremultipliedColorU8,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
) {
    // Fill a dummy array with `tail` values. `tail` is always in a 1..STAGE_WIDTH-1 range.
    // This way we can reuse the `load_8888__` method and remove any branches.
    let mut data = [PremultipliedColorU8::TRANSPARENT; STAGE_WIDTH];
    std::ptr::copy_nonoverlapping(ptr, data.as_mut_ptr(), tail);
    load_8888__(&data, r, g, b, a);
}

#[inline(always)]
unsafe fn load_8888__(
    data: &[PremultipliedColorU8; STAGE_WIDTH],
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
) {
    *r = U16x16::new([
        data[ 0].red() as u16, data[ 1].red() as u16, data[ 2].red() as u16, data[ 3].red() as u16,
        data[ 4].red() as u16, data[ 5].red() as u16, data[ 6].red() as u16, data[ 7].red() as u16,
        data[ 8].red() as u16, data[ 9].red() as u16, data[10].red() as u16, data[11].red() as u16,
        data[12].red() as u16, data[13].red() as u16, data[14].red() as u16, data[15].red() as u16,
    ]);

    *g = U16x16::new([
        data[ 0].green() as u16, data[ 1].green() as u16, data[ 2].green() as u16, data[ 3].green() as u16,
        data[ 4].green() as u16, data[ 5].green() as u16, data[ 6].green() as u16, data[ 7].green() as u16,
        data[ 8].green() as u16, data[ 9].green() as u16, data[10].green() as u16, data[11].green() as u16,
        data[12].green() as u16, data[13].green() as u16, data[14].green() as u16, data[15].green() as u16,
    ]);

    *b = U16x16::new([
        data[ 0].blue() as u16, data[ 1].blue() as u16, data[ 2].blue() as u16, data[ 3].blue() as u16,
        data[ 4].blue() as u16, data[ 5].blue() as u16, data[ 6].blue() as u16, data[ 7].blue() as u16,
        data[ 8].blue() as u16, data[ 9].blue() as u16, data[10].blue() as u16, data[11].blue() as u16,
        data[12].blue() as u16, data[13].blue() as u16, data[14].blue() as u16, data[15].blue() as u16,
    ]);

    *a = U16x16::new([
        data[ 0].alpha() as u16, data[ 1].alpha() as u16, data[ 2].alpha() as u16, data[ 3].alpha() as u16,
        data[ 4].alpha() as u16, data[ 5].alpha() as u16, data[ 6].alpha() as u16, data[ 7].alpha() as u16,
        data[ 8].alpha() as u16, data[ 9].alpha() as u16, data[10].alpha() as u16, data[11].alpha() as u16,
        data[12].alpha() as u16, data[13].alpha() as u16, data[14].alpha() as u16, data[15].alpha() as u16,
    ]);
}

#[inline(always)]
unsafe fn store_8888_(
    ptr: *mut PremultipliedColorU8,
    r: &U16x16, g: &U16x16, b: &U16x16, a: &U16x16,
) {
    let r = r.as_slice();
    let g = g.as_slice();
    let b = b.as_slice();
    let a = a.as_slice();

    *ptr.add( 0) = PremultipliedColorU8::from_rgba_unchecked(r[ 0] as u8, g[ 0] as u8, b[ 0] as u8, a[ 0] as u8);
    *ptr.add( 1) = PremultipliedColorU8::from_rgba_unchecked(r[ 1] as u8, g[ 1] as u8, b[ 1] as u8, a[ 1] as u8);
    *ptr.add( 2) = PremultipliedColorU8::from_rgba_unchecked(r[ 2] as u8, g[ 2] as u8, b[ 2] as u8, a[ 2] as u8);
    *ptr.add( 3) = PremultipliedColorU8::from_rgba_unchecked(r[ 3] as u8, g[ 3] as u8, b[ 3] as u8, a[ 3] as u8);
    *ptr.add( 4) = PremultipliedColorU8::from_rgba_unchecked(r[ 4] as u8, g[ 4] as u8, b[ 4] as u8, a[ 4] as u8);
    *ptr.add( 5) = PremultipliedColorU8::from_rgba_unchecked(r[ 5] as u8, g[ 5] as u8, b[ 5] as u8, a[ 5] as u8);
    *ptr.add( 6) = PremultipliedColorU8::from_rgba_unchecked(r[ 6] as u8, g[ 6] as u8, b[ 6] as u8, a[ 6] as u8);
    *ptr.add( 7) = PremultipliedColorU8::from_rgba_unchecked(r[ 7] as u8, g[ 7] as u8, b[ 7] as u8, a[ 7] as u8);
    *ptr.add( 8) = PremultipliedColorU8::from_rgba_unchecked(r[ 8] as u8, g[ 8] as u8, b[ 8] as u8, a[ 8] as u8);
    *ptr.add( 9) = PremultipliedColorU8::from_rgba_unchecked(r[ 9] as u8, g[ 9] as u8, b[ 9] as u8, a[ 9] as u8);
    *ptr.add(10) = PremultipliedColorU8::from_rgba_unchecked(r[10] as u8, g[10] as u8, b[10] as u8, a[10] as u8);
    *ptr.add(11) = PremultipliedColorU8::from_rgba_unchecked(r[11] as u8, g[11] as u8, b[11] as u8, a[11] as u8);
    *ptr.add(12) = PremultipliedColorU8::from_rgba_unchecked(r[12] as u8, g[12] as u8, b[12] as u8, a[12] as u8);
    *ptr.add(13) = PremultipliedColorU8::from_rgba_unchecked(r[13] as u8, g[13] as u8, b[13] as u8, a[13] as u8);
    *ptr.add(14) = PremultipliedColorU8::from_rgba_unchecked(r[14] as u8, g[14] as u8, b[14] as u8, a[14] as u8);
    *ptr.add(15) = PremultipliedColorU8::from_rgba_unchecked(r[15] as u8, g[15] as u8, b[15] as u8, a[15] as u8);
}

#[inline(always)]
unsafe fn store_8888_tail_(
    tail: usize, ptr: *mut PremultipliedColorU8,
    r: &U16x16, g: &U16x16, b: &U16x16, a: &U16x16,
) {
    let r = r.as_slice();
    let g = g.as_slice();
    let b = b.as_slice();
    let a = a.as_slice();

    // This is better than `for i in 0..tail`, because this way the compiler
    // knows that we have only 8 steps and slices access is guarantee to be valid.
    // This removes bounds checking and a possible panic call.
    for i in 0..STAGE_WIDTH {
        *ptr.add(i) = PremultipliedColorU8::from_rgba_unchecked(
            r[i] as u8, g[i] as u8, b[i] as u8, a[i] as u8,
        );

        if i + 1 == tail {
            break;
        }
    }
}

#[inline(always)]
fn div255(v: U16x16) -> U16x16 {
    (v + U16x16::splat(255)) / U16x16::splat(256)
}

#[inline(always)]
fn inv(v: U16x16) -> U16x16 {
    U16x16::splat(255) - v
}

#[inline(always)]
fn from_float(f: f32) -> U16x16 {
    U16x16::splat((f * 255.0 + 0.5) as u16)
}

#[inline(always)]
fn lerp(from: U16x16, to: U16x16, t: U16x16) -> U16x16 {
    div255(from * inv(t) + to * t)
}
