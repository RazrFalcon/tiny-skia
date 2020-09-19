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
we are still 40-60% behind Skia built for Haswell.
*/

use std::ffi::c_void;

use crate::{ScreenIntRect, PremultipliedColorU8, Transform};

use crate::wide::{F32x4, U16x16, F32x16};

const STAGE_WIDTH: usize = 16;

type StageFn = unsafe fn(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
);

// Must be in the same order as raster_pipeline::Stage
pub const STAGES: &[StageFn; super::STAGES_COUNT] = &[
    move_source_to_destination,
    move_destination_to_source,
    null_fn, // Clamp0
    null_fn, // ClampA
    premultiply,
    uniform_color,
    seed_shader,
    load_dst,
    store,
    null_fn, // Gather
    scale_1_float,
    lerp_1_float,
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
    transform_translate,
    transform_scale_translate,
    transform_2x3,
    null_fn, // RepeatX
    null_fn, // RepeatY
    null_fn, // Bilinear
    null_fn, // Bicubic
    pad_x1,
    reflect_x1,
    repeat_x1,
    gradient,
    evenly_spaced_2_stop_gradient,
    xy_to_radius,
    null_fn, // XYTo2PtConicalFocalOnCircle
    null_fn, // XYTo2PtConicalWellBehaved
    null_fn, // XYTo2PtConicalGreater
    null_fn, // Mask2PtConicalDegenerates
    null_fn, // ApplyVectorMask
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

unsafe fn premultiply(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg
    : &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    *r = div255(*r * *a);
    *g = div255(*g * *a);
    *b = div255(*b * *a);

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn uniform_color(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &super::UniformColorCtx = &*(*program.add(1)).cast();
    *r = U16x16::splat(ctx.rgba[0]);
    *g = U16x16::splat(ctx.rgba[1]);
    *b = U16x16::splat(ctx.rgba[2]);
    *a = U16x16::splat(ctx.rgba[3]);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn seed_shader(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let iota = F32x16([
        F32x4::new( 0.5,  1.5,  2.5,  3.5),
        F32x4::new( 4.5,  5.5,  6.5,  7.5),
        F32x4::new( 8.5,  9.5, 10.5, 11.5),
        F32x4::new(12.5, 13.5, 14.5, 15.5),
    ]);

    let x = F32x16::splat(dx as f32) + iota;
    let y = F32x16::splat(dy as f32 + 0.5);
    split(&x, r, g);
    split(&y, b, a);

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

pub unsafe fn load_dst(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &super::MemoryCtx = &*(*program.add(1)).cast();
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
    let ctx: &super::MemoryCtx = &*(*program.add(1)).cast();
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
    let ctx: &super::MemoryCtx = &*(*program.add(1)).cast();
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
    let ctx: &super::MemoryCtx = &*(*program.add(1)).cast();
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
        + (s+s).packed_le(&sa).if_then_else(
            U16x16::splat(2) * s * d,
            sa * da - U16x16::splat(2) * (sa-s)*(da-d)
        )
    )
});

blend_fn2!(overlay, |s: U16x16, d: U16x16, sa, da| {
    div255(s * inv(da) + d * inv(sa)
        + (d+d).packed_le(&da).if_then_else(
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
    let ctx: &super::MemoryCtx = &*(*program.add(1)).cast();
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
    let ctx: &super::MemoryCtx = &*(*program.add(1)).cast();
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

unsafe fn transform_translate(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ts: &Transform = &*(*program.add(1)).cast();
    let (tx, ty) = ts.get_translate();

    let x = join(r, g);
    let y = join(b, a);

    let nx = x + F32x16::splat(tx);
    let ny = y + F32x16::splat(ty);

    split(&nx, r, g);
    split(&ny, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn transform_scale_translate(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ts: &Transform = &*(*program.add(1)).cast();
    let (sx, sy) = ts.get_scale();
    let (tx, ty) = ts.get_translate();

    let x = join(r, g);
    let y = join(b, a);

    let nx = mad(x, F32x16::splat(sx), F32x16::splat(tx));
    let ny = mad(y, F32x16::splat(sy), F32x16::splat(ty));

    split(&nx, r, g);
    split(&ny, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn transform_2x3(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ts: &Transform = &*(*program.add(1)).cast();
    let (sx, ky, kx, sy, tx, ty) = ts.get_row();

    let x = join(r, g);
    let y = join(b, a);

    let nx = mad(x, F32x16::splat(sx), mad(y, F32x16::splat(kx), F32x16::splat(tx)));
    let ny = mad(x, F32x16::splat(ky), mad(y, F32x16::splat(sy), F32x16::splat(ty)));

    split(&nx, r, g);
    split(&ny, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn pad_x1(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let x = join(r, g);
    let x = x.normalize();
    split(&x, r, g);

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn reflect_x1(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let x = join(r, g);
    let two = |x| x + x;
    let x = (
        (x - F32x16::splat(1.0))
        - two(((x - F32x16::splat(1.0)) * F32x16::splat(0.5)).floor())
        - F32x16::splat(1.0)
    ).abs().normalize();
    split(&x, r, g);

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn repeat_x1(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let x = join(r, g);
    let x = (x - x.floor()).normalize();
    split(&x, r, g);

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn gradient(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &super::GradientCtx = &*(*program.add(1)).cast();

    // N.B. The loop starts at 1 because idx 0 is the color to use before the first stop.
    let t = join(r, g);
    let mut idx = U16x16::splat(0);
    for i in 1..ctx.len {
        let tt = ctx.t_values[i].get();
        idx.0[ 0] += (t.0[0].x() >= tt) as u16;
        idx.0[ 1] += (t.0[0].y() >= tt) as u16;
        idx.0[ 2] += (t.0[0].z() >= tt) as u16;
        idx.0[ 3] += (t.0[0].w() >= tt) as u16;
        idx.0[ 4] += (t.0[1].x() >= tt) as u16;
        idx.0[ 5] += (t.0[1].y() >= tt) as u16;
        idx.0[ 6] += (t.0[1].z() >= tt) as u16;
        idx.0[ 7] += (t.0[1].w() >= tt) as u16;
        idx.0[ 8] += (t.0[2].x() >= tt) as u16;
        idx.0[ 9] += (t.0[2].y() >= tt) as u16;
        idx.0[10] += (t.0[2].z() >= tt) as u16;
        idx.0[11] += (t.0[2].w() >= tt) as u16;
        idx.0[12] += (t.0[3].x() >= tt) as u16;
        idx.0[13] += (t.0[3].y() >= tt) as u16;
        idx.0[14] += (t.0[3].z() >= tt) as u16;
        idx.0[15] += (t.0[3].w() >= tt) as u16;
    }
    gradient_lookup(ctx, &idx, t, r, g, b, a);

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn evenly_spaced_2_stop_gradient(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let ctx: &super::EvenlySpaced2StopGradientCtx = &*(*program.add(1)).cast();

    let t = join(r, g);
    round_f32_to_u16(
        mad(t, F32x16::splat(ctx.factor.r), F32x16::splat(ctx.bias.r)),
        mad(t, F32x16::splat(ctx.factor.g), F32x16::splat(ctx.bias.g)),
        mad(t, F32x16::splat(ctx.factor.b), F32x16::splat(ctx.bias.b)),
        mad(t, F32x16::splat(ctx.factor.a), F32x16::splat(ctx.bias.a)),
        r, g, b, a,
    );

    let next: StageFn = *program.add(2).cast();
    next(tail, program.add(2), dx,dy, r,g,b,a, dr,dg,db,da);
}

unsafe fn xy_to_radius(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
    dr: &mut U16x16, dg: &mut U16x16, db: &mut U16x16, da: &mut U16x16,
) {
    let x = join(r, g);
    let y = join(b, a);
    let x = (x*x + y*y).sqrt();
    split(&x, r, g);
    split(&y, b, a);

    let next: StageFn = *program.add(1).cast();
    next(tail, program.add(1), dx,dy, r,g,b,a, dr,dg,db,da);
}

// We are using u16 for index, not u32 as Skia, to simplify the code a bit.
// The gradient creation code will not allow that many stops anyway.
fn gradient_lookup(
    ctx: &super::GradientCtx, idx: &U16x16, t: F32x16,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
) {
    macro_rules! gather {
        ($d:expr, $c:ident) => {
            // Surprisingly, but bound checking doesn't affect the performance.
            // And since `idx` can contain any number, we should leave it in place.
            F32x16([
                F32x4::new(
                    $d[idx.0[ 0] as usize].$c,
                    $d[idx.0[ 1] as usize].$c,
                    $d[idx.0[ 2] as usize].$c,
                    $d[idx.0[ 3] as usize].$c,
                ),
                F32x4::new(
                    $d[idx.0[ 4] as usize].$c,
                    $d[idx.0[ 5] as usize].$c,
                    $d[idx.0[ 6] as usize].$c,
                    $d[idx.0[ 7] as usize].$c,
                ),
                F32x4::new(
                    $d[idx.0[ 8] as usize].$c,
                    $d[idx.0[ 9] as usize].$c,
                    $d[idx.0[10] as usize].$c,
                    $d[idx.0[11] as usize].$c,
                ),
                F32x4::new(
                    $d[idx.0[12] as usize].$c,
                    $d[idx.0[13] as usize].$c,
                    $d[idx.0[14] as usize].$c,
                    $d[idx.0[15] as usize].$c,
                ),
            ])
        };
    }

    let fr = gather!(&ctx.factors, r);
    let fg = gather!(&ctx.factors, g);
    let fb = gather!(&ctx.factors, b);
    let fa = gather!(&ctx.factors, a);

    let br = gather!(&ctx.biases, r);
    let bg = gather!(&ctx.biases, g);
    let bb = gather!(&ctx.biases, b);
    let ba = gather!(&ctx.biases, a);

    round_f32_to_u16(
        mad(t, fr, br),
        mad(t, fg, bg),
        mad(t, fb, bb),
        mad(t, fa, ba),
        r, g, b, a,
    );
}

#[inline(always)]
fn round_f32_to_u16(
    rf: F32x16, gf: F32x16, bf: F32x16, af: F32x16,
    r: &mut U16x16, g: &mut U16x16, b: &mut U16x16, a: &mut U16x16,
) {
    // TODO: may produce a slightly different result to Skia
    //       affects the two_stops_linear_mirror test

    let rf = rf.normalize() * F32x16::splat(255.0) + F32x16::splat(0.5);
    let gf = gf.normalize() * F32x16::splat(255.0) + F32x16::splat(0.5);
    let bf = bf.normalize() * F32x16::splat(255.0) + F32x16::splat(0.5);
    let af = af * F32x16::splat(255.0) + F32x16::splat(0.5);

    rf.save_to_u16x16(r);
    gf.save_to_u16x16(g);
    bf.save_to_u16x16(b);
    af.save_to_u16x16(a);
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
    *r = U16x16([
        data[ 0].red() as u16, data[ 1].red() as u16, data[ 2].red() as u16, data[ 3].red() as u16,
        data[ 4].red() as u16, data[ 5].red() as u16, data[ 6].red() as u16, data[ 7].red() as u16,
        data[ 8].red() as u16, data[ 9].red() as u16, data[10].red() as u16, data[11].red() as u16,
        data[12].red() as u16, data[13].red() as u16, data[14].red() as u16, data[15].red() as u16,
    ]);

    *g = U16x16([
        data[ 0].green() as u16, data[ 1].green() as u16, data[ 2].green() as u16, data[ 3].green() as u16,
        data[ 4].green() as u16, data[ 5].green() as u16, data[ 6].green() as u16, data[ 7].green() as u16,
        data[ 8].green() as u16, data[ 9].green() as u16, data[10].green() as u16, data[11].green() as u16,
        data[12].green() as u16, data[13].green() as u16, data[14].green() as u16, data[15].green() as u16,
    ]);

    *b = U16x16([
        data[ 0].blue() as u16, data[ 1].blue() as u16, data[ 2].blue() as u16, data[ 3].blue() as u16,
        data[ 4].blue() as u16, data[ 5].blue() as u16, data[ 6].blue() as u16, data[ 7].blue() as u16,
        data[ 8].blue() as u16, data[ 9].blue() as u16, data[10].blue() as u16, data[11].blue() as u16,
        data[12].blue() as u16, data[13].blue() as u16, data[14].blue() as u16, data[15].blue() as u16,
    ]);

    *a = U16x16([
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
    // knows that we have only 16 steps and slices access is guarantee to be valid.
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

#[inline(always)]
fn split(v: &F32x16, lo: &mut U16x16, hi: &mut U16x16) {
    const U16X16_SIZEOF: usize = std::mem::size_of::<U16x16>();

    unsafe {
        let v_data = v.0.as_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(
            v_data,
            lo.as_mut_slice().as_mut_ptr() as *mut u8,
            U16X16_SIZEOF,
        );
        std::ptr::copy_nonoverlapping(
            v_data.add(U16X16_SIZEOF),
            hi.as_mut_slice().as_mut_ptr() as *mut u8,
            U16X16_SIZEOF,
        );
    }
}

#[inline(always)]
fn join(lo: &U16x16, hi: &U16x16) -> F32x16 {
    const U16X16_SIZEOF: usize = std::mem::size_of::<U16x16>();

    let mut v = F32x16::default();
    unsafe {
        let v_data = v.0.as_mut_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(
            lo.as_slice().as_ptr() as *const u8,
            v_data,
            U16X16_SIZEOF,
        );
        std::ptr::copy_nonoverlapping(
            hi.as_slice().as_ptr() as *const u8,
            v_data.add(U16X16_SIZEOF),
            U16X16_SIZEOF,
        );
    }

    v
}

#[inline(always)]
fn mad(f: F32x16, m: F32x16, a: F32x16) -> F32x16 {
    f * m + a
}
