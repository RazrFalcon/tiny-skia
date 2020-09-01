// Copyright 2018 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/*!
A high precision raster pipeline implementation.

Unlike lowp, this one implements all stages (not right now, but will eventually).

Just like Skia, this pipeline is implemented using F32x4.
Skia also supports F32x8 on modern CPUs, but we're not at the moment.

For some reason, we are almost 2x slower. Maybe because Skia uses clang's vector extensions
and we're using a manual implementation.
*/

use std::ffi::c_void;

use crate::{ScreenIntRect, PremultipliedColorU8};

use crate::raster_pipeline::{self, STAGES_COUNT};
use crate::wide::{I32x4, F32x4};

const STAGE_WIDTH: usize = 4;

type StageFn = unsafe fn(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
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
    unbounded_uniform_color,
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
    scale_1_float,
    null_fn, // ScaleNative,
    null_fn, // LerpU8,
    lerp_1_float,
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
    color_burn,
    color_dodge,
    darken,
    difference,
    exclusion,
    hard_light,
    lighten,
    overlay,
    soft_light,
    hue,
    saturation,
    color,
    luminosity,
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
    let mut  r = F32x4::default();
    let mut  g = F32x4::default();
    let mut  b = F32x4::default();
    let mut  a = F32x4::default();
    let mut dr = F32x4::default();
    let mut dg = F32x4::default();
    let mut db = F32x4::default();
    let mut da = F32x4::default();

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
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
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
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
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
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::UniformColorCtx = &*(*program.add(1)).cast();
    *r = F32x4::splat(ctx.r);
    *g = F32x4::splat(ctx.g);
    *b = F32x4::splat(ctx.b);
    *a = F32x4::splat(ctx.a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

// Identical to uniform_color.
unsafe fn unbounded_uniform_color(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::UniformColorCtx = &*(*program.add(1)).cast();
    *r = F32x4::splat(ctx.r);
    *g = F32x4::splat(ctx.g);
    *b = F32x4::splat(ctx.b);
    *a = F32x4::splat(ctx.a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn load_dst(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_(ptr, dr, dg, db, da);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn load_dst_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_tail_(tail, ptr, dr, dg, db, da);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn store(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    store_8888_(ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn store_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    store_8888_tail_(tail, ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn scale_1_float(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let c: f32 = *(*program.add(1)).cast();
    let c = F32x4::splat(c);
    *r = *r * c;
    *g = *g * c;
    *b = *b * c;
    *a = *a * c;

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn lerp_1_float(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let c: f32 = *(*program.add(1)).cast();
    let c = F32x4::splat(c);
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
            r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
            dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
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

blend_fn!(clear,            |_, _,  _,  _| F32x4::default());
blend_fn!(source_atop,      |s, d, sa, da| s * da + d * inv(sa));
blend_fn!(destination_atop, |s, d, sa, da| d * sa + s * inv(da));
blend_fn!(source_in,        |s, _,  _, da| s * da);
blend_fn!(destination_in,   |_, d, sa,  _| d * sa);
blend_fn!(source_out,       |s, _,  _, da| s * inv(da));
blend_fn!(destination_out,  |_, d, sa,  _| d * inv(sa));
blend_fn!(source_over,      |s, d, sa,  _| mad(d, inv(sa), s));
blend_fn!(destination_over, |s, d,  _, da| mad(s, inv(da), d));
blend_fn!(modulate,         |s, d,  _,  _| s * d);
blend_fn!(multiply,         |s, d, sa, da| s * inv(da) + d * inv(sa) + s * d);
blend_fn!(screen,           |s, d,  _,  _| s + d - s * d);
blend_fn!(xor,              |s, d, sa, da| s * inv(da) + d * inv(sa));

// Wants a type for some reason.
blend_fn!(plus, |s: F32x4, d, _, _| (s + d).min(F32x4::splat(1.0)));

macro_rules! blend_fn2 {
    ($name:ident, $f:expr) => {
        unsafe fn $name(
            tail: usize, program: *const *const c_void, dx: usize, dy: usize,
            r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
            dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
        ) {
            // The same logic applied to color, and source_over for alpha.
            *r = $f(*r, *dr, *a, *da);
            *g = $f(*g, *dg, *a, *da);
            *b = $f(*b, *db, *a, *da);
            *a = mad(*da, inv(*a), *a);

            let next: StageFn = *program.add(1).cast();
            next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
        }
    };
}

blend_fn2!(darken,      |s: F32x4, d, sa, da| s + d - (s * da).max(d * sa));
blend_fn2!(lighten,     |s: F32x4, d, sa, da| s + d - (s * da).min(d * sa));
blend_fn2!(difference,  |s: F32x4, d, sa, da| s + d - two((s * da).min(d * sa)));
blend_fn2!(exclusion,   |s: F32x4, d,  _,  _| s + d - two(s * d));

blend_fn2!(color_burn, |s: F32x4, d: F32x4, sa, da|
    d.packed_eq(da).if_then_else(
        d + s * inv(da),
        s.packed_eq(F32x4::default()).if_then_else(
            d * inv(sa),
            sa * (da - da.min((da - d) * sa * s.approx_recip())) + s * inv(da) + d * inv(sa)
        )
    )
);

blend_fn2!(color_dodge, |s: F32x4, d: F32x4, sa, da|
    d.packed_eq(F32x4::default()).if_then_else(
        s * inv(da),
        s.packed_eq(sa).if_then_else(
            s + d * inv(sa),
            sa * da.min((d * sa) * (sa - s).approx_recip()) + s * inv(da) + d * inv(sa)
        )
    )
);

blend_fn2!(hard_light, |s: F32x4, d: F32x4, sa, da|
    s * inv(da) + d * inv(sa) + two(s).packed_le(sa).if_then_else(
        two(s * d),
        sa * da - two((da - d) * (sa - s))
    )
);

blend_fn2!(overlay, |s: F32x4, d: F32x4, sa, da|
    s * inv(da) + d * inv(sa) + two(d).packed_le(da).if_then_else(
        two(s * d),
        sa * da - two((da - d) * (sa - s))
    )
);

blend_fn2!(soft_light, |s: F32x4, d: F32x4, sa: F32x4, da: F32x4| {
    let m  = da.packed_gt(F32x4::default()).if_then_else(d / da, F32x4::default());
    let s2 = two(s);
    let m4 = two(two(m));

    // The logic forks three ways:
    //    1. dark src?
    //    2. light src, dark dst?
    //    3. light src, light dst?
    let dark_src = d * (sa + (s2 - sa) * (F32x4::splat(1.0) - m));
    let dark_dst = (m4 * m4 + m4) * (m - F32x4::splat(1.0)) + F32x4::splat(7.0) * m;
    let lite_dst = m.approx_recip_sqrt().approx_recip() - m;
    let lite_src = d * sa + da * (s2 - sa)
        * two(two(d)).packed_le(da).if_then_else(dark_dst, lite_dst); // 2 or 3?

    s*inv(da) + d*inv(sa) + s2.packed_le(sa).if_then_else(dark_src, lite_src) // 1 or (2 or 3)?
});

// We're basing our implementation of non-separable blend modes on
//   https://www.w3.org/TR/compositing-1/#blendingnonseparable.
// and
//   https://www.khronos.org/registry/OpenGL/specs/es/3.2/es_spec_3.2.pdf
// They're equivalent, but ES' math has been better simplified.
//
// Anything extra we add beyond that is to make the math work with premul inputs.

macro_rules! blend_fn3 {
    ($name:ident, $f:expr) => {
        unsafe fn $name(
            tail: usize, program: *const *const c_void, dx: usize, dy: usize,
            r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
            dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
        ) {
            let (tr, tg, tb, ta) = $f(*r, *g, *b, *a, *dr, *dg, *db, *da);
            *r = tr;
            *g = tg;
            *b = tb;
            *a = ta;

            let next: StageFn = *program.add(1).cast();
            next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
        }
    };
}

blend_fn3!(hue, hue_k);

#[inline(always)]
fn hue_k(
    r: F32x4, g: F32x4, b: F32x4, a: F32x4,
    dr: F32x4, dg: F32x4, db: F32x4, da: F32x4,
) -> (F32x4, F32x4, F32x4, F32x4) {
    let rr = &mut (r * a);
    let gg = &mut (g * a);
    let bb = &mut (b * a);

    set_sat(rr, gg, bb, sat(dr, dg, db) * a);
    set_lum(rr, gg, bb, lum(dr, dg, db) * a);
    clip_color(rr, gg, bb, a * da);

    let r = r * inv(da) + dr * inv(a) + *rr;
    let g = g * inv(da) + dg * inv(a) + *gg;
    let b = b * inv(da) + db * inv(a) + *bb;
    let a = a + da - a * da;

    (r, g, b, a)
}

blend_fn3!(saturation, saturation_k);

#[inline(always)]
fn saturation_k(
    r: F32x4, g: F32x4, b: F32x4, a: F32x4,
    dr: F32x4, dg: F32x4, db: F32x4, da: F32x4,
) -> (F32x4, F32x4, F32x4, F32x4) {
    let rr = &mut (dr * a);
    let gg = &mut (dg * a);
    let bb = &mut (db * a);

    set_sat(rr, gg, bb, sat(r, g, b) * da);
    set_lum(rr, gg, bb, lum(dr, dg, db) * a); // (This is not redundant.)
    clip_color(rr, gg, bb, a * da);

    let r = r * inv(da) + dr * inv(a) + *rr;
    let g = g * inv(da) + dg * inv(a) + *gg;
    let b = b * inv(da) + db * inv(a) + *bb;
    let a = a + da - a * da;

    (r, g, b, a)
}

blend_fn3!(color, color_k);

#[inline(always)]
fn color_k(
    r: F32x4, g: F32x4, b: F32x4, a: F32x4,
    dr: F32x4, dg: F32x4, db: F32x4, da: F32x4,
) -> (F32x4, F32x4, F32x4, F32x4) {
    let rr = &mut (r * da);
    let gg = &mut (g * da);
    let bb = &mut (b * da);

    set_lum(rr, gg, bb, lum(dr, dg, db) * a);
    clip_color(rr, gg, bb, a * da);

    let r = r * inv(da) + dr * inv(a) + *rr;
    let g = g * inv(da) + dg * inv(a) + *gg;
    let b = b * inv(da) + db * inv(a) + *bb;
    let a = a + da - a * da;

    (r, g, b, a)
}

blend_fn3!(luminosity, luminosity_k);

#[inline(always)]
fn luminosity_k(
    r: F32x4, g: F32x4, b: F32x4, a: F32x4,
    dr: F32x4, dg: F32x4, db: F32x4, da: F32x4,
) -> (F32x4, F32x4, F32x4, F32x4) {
    let rr = &mut (dr * a);
    let gg = &mut (dg * a);
    let bb = &mut (db * a);

    set_lum(rr, gg, bb, lum(r, g, b) * da);
    clip_color(rr, gg, bb, a * da);

    let r = r * inv(da) + dr * inv(a) + *rr;
    let g = g * inv(da) + dg * inv(a) + *gg;
    let b = b * inv(da) + db * inv(a) + *bb;
    let a = a + da - a * da;

    (r, g, b, a)
}

#[inline(always)]
fn sat(r: F32x4, g: F32x4, b: F32x4) -> F32x4 {
    r.max(g.max(b)) - r.min(g.min(b))
}

#[inline(always)]
fn lum(r: F32x4, g: F32x4, b: F32x4) -> F32x4 {
    r * F32x4::splat(0.30) + g * F32x4::splat(0.59) + b * F32x4::splat(0.11)
}

#[inline(always)]
fn set_sat(r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, s: F32x4) {
    let mn  = r.min(g.min(*b));
    let mx  = r.max(g.max(*b));
    let sat = mx - mn;

    // Map min channel to 0, max channel to s, and scale the middle proportionally.
    let scale = |c| sat.packed_eq(F32x4::default())
                       .if_then_else(F32x4::default(), (c - mn) * s / sat);

    *r = scale(*r);
    *g = scale(*g);
    *b = scale(*b);
}

#[inline(always)]
fn set_lum(r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, l: F32x4) {
    let diff = l - lum(*r, *g, *b);
    *r = *r + diff;
    *g = *g + diff;
    *b = *b + diff;
}

#[inline(always)]
fn clip_color(r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: F32x4) {
    let mn = r.min(g.min(*b));
    let mx = r.max(g.max(*b));
    let l  = lum(*r, *g, *b);

    let clip = |mut c| {
        c = mx.packed_ge(F32x4::default()).if_then_else(c, l + (c - l) * l / (l - mn));
        c = mx.packed_gt(a).if_then_else(l + (c - l) * (a - l) / (mx - l), c);
        c = c.max(F32x4::default()); // Sometimes without this we may dip just a little negative.
        c
    };

    *r = clip(*r);
    *g = clip(*g);
    *b = clip(*b);
}

pub unsafe fn source_over_rgba(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_(ptr, dr, dg, db, da);
    *r = mad(*dr, inv(*a), *r);
    *g = mad(*dg, inv(*a), *g);
    *b = mad(*db, inv(*a), *b);
    *a = mad(*da, inv(*a), *a);
    store_8888_(ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn source_over_rgba_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &raster_pipeline::MemoryCtx = &*(*program.add(1)).cast();
    let ptr = ctx.ptr_at_xy::<PremultipliedColorU8>(dx, dy);
    load_8888_tail_(tail, ptr, dr, dg, db, da);
    *r = mad(*dr, inv(*a), *r);
    *g = mad(*dg, inv(*a), *g);
    *b = mad(*db, inv(*a), *b);
    *a = mad(*da, inv(*a), *a);
    store_8888_tail_(tail, ptr, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn just_return(
    _: usize, _: *const *const c_void, _: usize, _: usize,
    _: &mut F32x4, _: &mut F32x4, _: &mut F32x4, _: &mut F32x4,
    _: &mut F32x4, _: &mut F32x4, _: &mut F32x4, _: &mut F32x4,
) {
    // Ends the loop.
}

pub unsafe fn null_fn(
    _: usize, _: *const *const c_void, _: usize, _: usize,
    _: &mut F32x4, _: &mut F32x4, _: &mut F32x4, _: &mut F32x4,
    _: &mut F32x4, _: &mut F32x4, _: &mut F32x4, _: &mut F32x4,
) {
    // Just for unsupported functions in STAGES.
}

#[inline(always)]
unsafe fn load_8888_(
    ptr: *const PremultipliedColorU8,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
) {
    // Cast a data pointer to a fixed size array.
    let data = &*(ptr as *const [PremultipliedColorU8; STAGE_WIDTH]);
    load_8888__(data, r, g, b, a);
}

#[inline(always)]
unsafe fn load_8888_tail_(
    tail: usize, ptr: *const PremultipliedColorU8,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
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
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
) {
    // Surprisingly, `f32 * FACTOR` is way faster than `F32x4 * F32x4::splat(FACTOR)`.

    const FACTOR: f32 = 1.0 / 255.0;

    *r = F32x4::new(
        data[0].red() as f32 * FACTOR, data[1].red() as f32 * FACTOR,
        data[2].red() as f32 * FACTOR, data[3].red() as f32 * FACTOR,
    );

    *g = F32x4::new(
        data[0].green() as f32 * FACTOR, data[1].green() as f32 * FACTOR,
        data[2].green() as f32 * FACTOR, data[3].green() as f32 * FACTOR,
    );

    *b = F32x4::new(
        data[0].blue() as f32 * FACTOR, data[1].blue() as f32 * FACTOR,
        data[2].blue() as f32 * FACTOR, data[3].blue() as f32 * FACTOR,
    );

    *a = F32x4::new(
        data[0].alpha() as f32 * FACTOR, data[1].alpha() as f32 * FACTOR,
        data[2].alpha() as f32 * FACTOR, data[3].alpha() as f32 * FACTOR,
    );
}

#[inline(always)]
unsafe fn store_8888_(
    ptr: *mut PremultipliedColorU8,
    r: &F32x4, g: &F32x4, b: &F32x4, a: &F32x4,
) {
    let r = unnorm(r);
    let g = unnorm(g);
    let b = unnorm(b);
    let a = unnorm(a);

    let conv = |rr, gg, bb, aa|
        PremultipliedColorU8::from_rgba_unchecked(rr as u8, gg as u8, bb as u8, aa as u8);

    *ptr.add(0) = conv(r.x(), g.x(), b.x(), a.x());
    *ptr.add(1) = conv(r.y(), g.y(), b.y(), a.y());
    *ptr.add(2) = conv(r.z(), g.z(), b.z(), a.z());
    *ptr.add(3) = conv(r.w(), g.w(), b.w(), a.w());
}

#[inline(always)]
unsafe fn store_8888_tail_(
    tail: usize, ptr: *mut PremultipliedColorU8,
    r: &F32x4, g: &F32x4, b: &F32x4, a: &F32x4,
) {
    let r = unnorm(r);
    let g = unnorm(g);
    let b = unnorm(b);
    let a = unnorm(a);

    let r = r.as_slice();
    let g = g.as_slice();
    let b = b.as_slice();
    let a = a.as_slice();

    // This is better than `for i in 0..tail`, because this way the compiler
    // knows that we have only 4 steps and slices access is guarantee to be valid.
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
fn unnorm(v: &F32x4) -> I32x4 {
    (v.max(F32x4::default()).min(F32x4::splat(1.0)) * F32x4::splat(255.0)).to_i32x4()
}

#[inline(always)]
fn inv(v: F32x4) -> F32x4 {
    F32x4::splat(1.0) - v
}

#[inline(always)]
fn two(v: F32x4) -> F32x4 {
    v + v
}

#[inline(always)]
fn mad(f: F32x4, m: F32x4, a: F32x4) -> F32x4 {
    f * m + a
}

#[inline(always)]
fn lerp(from: F32x4, to: F32x4, t: F32x4) -> F32x4 {
    mad(to - from, t, from)
}
