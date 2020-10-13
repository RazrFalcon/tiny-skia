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

use crate::{ScreenIntRect, PremultipliedColorU8, Transform, SpreadMode};

use crate::wide::{I32x4, U32x4, F32x4};

pub const STAGE_WIDTH: usize = 4;

type StageFn = fn(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
);

// Must be in the same order as raster_pipeline::Stage
pub const STAGES: &[StageFn; super::STAGES_COUNT] = &[
    move_source_to_destination,
    move_destination_to_source,
    clamp_0,
    clamp_a,
    premultiply,
    uniform_color,
    seed_shader,
    load_dst,
    store,
    gather,
    scale_u8,
    lerp_u8,
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
    transform,
    reflect_x,
    reflect_y,
    repeat_x,
    repeat_y,
    bilinear,
    bicubic,
    pad_x1,
    reflect_x1,
    repeat_x1,
    gradient,
    evenly_spaced_2_stop_gradient,
    xy_to_radius,
    xy_to_2pt_conical_focal_on_circle,
    xy_to_2pt_conical_well_behaved,
    xy_to_2pt_conical_greater,
    mask_2pt_conical_degenerates,
    apply_vector_mask,
];

pub fn fn_ptr(f: StageFn) -> *const c_void {
    f as *const () as *const c_void
}

#[inline(never)]
pub fn start(
    program: *const *const c_void,
    tail_program: *const *const c_void,
    rect: &ScreenIntRect,
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

fn move_source_to_destination(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *dr = *r;
    *dg = *g;
    *db = *b;
    *da = *a;

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn premultiply(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *r = *r * *a;
    *g = *g * *a;
    *b = *b * *a;

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn move_destination_to_source(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *r = *dr;
    *g = *dg;
    *b = *db;
    *a = *da;

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn clamp_0(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *r = r.max(F32x4::default());
    *g = g.max(F32x4::default());
    *b = b.max(F32x4::default());
    *a = a.max(F32x4::default());

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn clamp_a(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *r = r.min(F32x4::splat(1.0));
    *g = g.min(F32x4::splat(1.0));
    *b = b.min(F32x4::splat(1.0));
    *a = a.min(F32x4::splat(1.0));

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn uniform_color(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::UniformColorCtx = cast_stage(program);
    *r = F32x4::splat(ctx.r);
    *g = F32x4::splat(ctx.g);
    *b = F32x4::splat(ctx.b);
    *a = F32x4::splat(ctx.a);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn seed_shader(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let iota = F32x4::new(0.5, 1.5, 2.5, 3.5);

    *r = F32x4::splat(dx as f32) + iota;
    *g = F32x4::splat(dy as f32 + 0.5);
    *b = F32x4::splat(1.0);
    *a = F32x4::default();

    *dr = F32x4::default();
    *dg = F32x4::default();
    *db = F32x4::default();
    *da = F32x4::default();

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

pub fn load_dst(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::PixelsCtx::from_program(program);
    load_8888(ctx.slice4_at_xy(dx, dy), dr, dg, db, da);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

pub fn load_dst_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::PixelsCtx::from_program(program);
    load_8888_tail(tail, ctx.slice_at_xy(dx, dy), dr, dg, db, da);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

pub fn store(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::PixelsCtx::from_program(program);
    store_8888(r, g, b, a, ctx.slice4_at_xy(dx, dy));

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

pub fn gather(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::GatherCtx = cast_stage(program);

    let ix = gather_ix(ctx, *r, *g);
    let data = unsafe { [
        *ctx.pixels.cast::<PremultipliedColorU8>().add(ix.x() as usize),
        *ctx.pixels.cast::<PremultipliedColorU8>().add(ix.y() as usize),
        *ctx.pixels.cast::<PremultipliedColorU8>().add(ix.z() as usize),
        *ctx.pixels.cast::<PremultipliedColorU8>().add(ix.w() as usize),
    ] };
    load_8888(&data, r, g, b, a);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

#[inline(always)]
fn gather_ix(ctx: &super::GatherCtx, mut x: F32x4, mut y: F32x4) -> U32x4 {
    // Exclusive -> inclusive.
    let w = ulp_sub(ctx.width.get() as f32);
    let h = ulp_sub(ctx.height.get() as f32);
    x = x.max(F32x4::default()).min(F32x4::splat(w));
    y = y.max(F32x4::default()).min(F32x4::splat(h));

    y.trunc().to_u32x4() * U32x4::splat(ctx.stride.get()) + x.trunc().to_u32x4()
}

#[inline(always)]
fn ulp_sub(v: f32) -> f32 {
    // Somewhat similar to v - f32::EPSILON
    unsafe { std::mem::transmute::<u32, f32>(std::mem::transmute::<f32, u32>(v) - 1) }
}

pub fn store_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::PixelsCtx::from_program(program);
    store_8888_tail(r, g, b, a, tail, ctx.slice_at_xy(dx, dy));

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn scale_u8(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::MaskCtx::from_program(program);

    // Load u8xTail and cast it to F32x4.
    let data = ctx.copy_at_xy(dx, dy, tail);
    let c = F32x4::new(data[0] as f32, data[1] as f32, 0.0, 0.0);
    let c = c / F32x4::splat(255.0);

    *r = *r * c;
    *g = *g * c;
    *b = *b * c;
    *a = *a * c;

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn lerp_u8(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::MaskCtx::from_program(program);

    // Load u8xTail and cast it to F32x4.
    let data = ctx.copy_at_xy(dx, dy, tail);
    let c = F32x4::new(data[0] as f32, data[1] as f32, 0.0, 0.0);
    let c = c / F32x4::splat(255.0);

    *r = lerp(*dr, *r, c);
    *g = lerp(*dg, *g, c);
    *b = lerp(*db, *b, c);
    *a = lerp(*da, *a, c);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn scale_1_float(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let c: f32 = *cast_stage(program);
    let c = F32x4::splat(c);
    *r = *r * c;
    *g = *g * c;
    *b = *b * c;
    *a = *a * c;

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn lerp_1_float(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let c: f32 = *cast_stage(program);
    let c = F32x4::splat(c);
    *r = lerp(*dr, *r, c);
    *g = lerp(*dg, *g, c);
    *b = lerp(*db, *b, c);
    *a = lerp(*da, *a, c);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

macro_rules! blend_fn {
    ($name:ident, $f:expr) => {
        fn $name(
            tail: usize, program: *const *const c_void, dx: usize, dy: usize,
            r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
            dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
        ) {
            *r = $f(*r, *dr, *a, *da);
            *g = $f(*g, *dg, *a, *da);
            *b = $f(*b, *db, *a, *da);
            *a = $f(*a, *da, *a, *da);

            next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
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
        fn $name(
            tail: usize, program: *const *const c_void, dx: usize, dy: usize,
            r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
            dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
        ) {
            // The same logic applied to color, and source_over for alpha.
            *r = $f(*r, *dr, *a, *da);
            *g = $f(*g, *dg, *a, *da);
            *b = $f(*b, *db, *a, *da);
            *a = mad(*da, inv(*a), *a);

            next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
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
        fn $name(
            tail: usize, program: *const *const c_void, dx: usize, dy: usize,
            r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
            dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
        ) {
            let (tr, tg, tb, ta) = $f(*r, *g, *b, *a, *dr, *dg, *db, *da);
            *r = tr;
            *g = tg;
            *b = tb;
            *a = ta;

            next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
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

pub fn source_over_rgba(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::PixelsCtx::from_program(program);
    let pixels = ctx.slice4_at_xy(dx, dy);
    load_8888(pixels, dr, dg, db, da);
    *r = mad(*dr, inv(*a), *r);
    *g = mad(*dg, inv(*a), *g);
    *b = mad(*db, inv(*a), *b);
    *a = mad(*da, inv(*a), *a);
    store_8888(r, g, b, a, pixels);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

pub fn source_over_rgba_tail(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx = super::PixelsCtx::from_program(program);
    let pixels = ctx.slice_at_xy(dx, dy);
    load_8888_tail(tail, pixels, dr, dg, db, da);
    *r = mad(*dr, inv(*a), *r);
    *g = mad(*dg, inv(*a), *g);
    *b = mad(*db, inv(*a), *b);
    *a = mad(*da, inv(*a), *a);
    store_8888_tail(r, g, b, a, tail, pixels);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn transform(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ts: &Transform = cast_stage(program);
    let (sx, ky, kx, sy, tx, ty) = ts.get_row();

    let tr = mad(*r, F32x4::splat(sx), mad(*g, F32x4::splat(kx), F32x4::splat(tx)));
    let tg = mad(*r, F32x4::splat(ky), mad(*g, F32x4::splat(sy), F32x4::splat(ty)));
    *r = tr;
    *g = tg;

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

// Tile x or y to [0,limit) == [0,limit - 1 ulp] (think, sampling from images).
// The gather stages will hard clamp the output of these stages to [0,limit)...
// we just need to do the basic repeat or mirroring.

fn reflect_x(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::TileCtx = cast_stage(program);
    *r = exclusive_reflect(*r, ctx.scale, ctx.inv_scale);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn reflect_y(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::TileCtx = cast_stage(program);
    *g = exclusive_reflect(*g, ctx.scale, ctx.inv_scale);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

#[inline(always)]
fn exclusive_reflect(v: F32x4, limit: f32, inv_limit: f32) -> F32x4 {
    let limit = F32x4::splat(limit);
    let inv_limit = F32x4::splat(inv_limit);
    ((v - limit) - (limit + limit) * ((v - limit) * (inv_limit * F32x4::splat(0.5))).floor() - limit).abs()
}

fn repeat_x(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::TileCtx = cast_stage(program);
    *r = exclusive_repeat(*r, ctx.scale, ctx.inv_scale);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn repeat_y(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::TileCtx = cast_stage(program);
    *g = exclusive_repeat(*g, ctx.scale, ctx.inv_scale);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

#[inline(always)]
fn exclusive_repeat(v: F32x4, limit: f32, inv_limit: f32) -> F32x4 {
    v - (v * F32x4::splat(inv_limit)).floor() * F32x4::splat(limit)
}

fn bilinear(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::SamplerCtx = cast_stage(program);

    let x = *r;
    let fx = (x + F32x4::splat(0.5)).fract();
    let y = *g;
    let fy = (y + F32x4::splat(0.5)).fract();
    let one = F32x4::splat(1.0);
    let wx = [one - fx, fx];
    let wy = [one - fy, fy];

    sampler_2x2(ctx, x, y, &wx, &wy, r, g, b, a);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn bicubic(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::SamplerCtx = cast_stage(program);

    let x = *r;
    let fx = (x + F32x4::splat(0.5)).fract();
    let y = *g;
    let fy = (y + F32x4::splat(0.5)).fract();
    let one = F32x4::splat(1.0);
    let wx = [bicubic_far(one - fx), bicubic_near(one - fx), bicubic_near(fx), bicubic_far(fx)];
    let wy = [bicubic_far(one - fy), bicubic_near(one - fy), bicubic_near(fy), bicubic_far(fy)];

    sampler_4x4(ctx, x, y, &wx, &wy, r, g, b, a);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

// In bicubic interpolation, the 16 pixels and +/- 0.5 and +/- 1.5 offsets from the sample
// pixel center are combined with a non-uniform cubic filter, with higher values near the center.
//
// We break this function into two parts, one for near 0.5 offsets and one for far 1.5 offsets.

#[inline(always)]
fn bicubic_near(t: F32x4) -> F32x4 {
    // 1/18 + 9/18t + 27/18t^2 - 21/18t^3 == t ( t ( -21/18t + 27/18) + 9/18) + 1/18
    mad(t, mad(t, mad(F32x4::splat(-21.0/18.0), t, F32x4::splat(27.0/18.0)), F32x4::splat(9.0/18.0)), F32x4::splat(1.0/18.0))
}

#[inline(always)]
fn bicubic_far(t: F32x4) -> F32x4 {
    // 0/18 + 0/18*t - 6/18t^2 + 7/18t^3 == t^2 (7/18t - 6/18)
    (t * t) * mad(F32x4::splat(7.0/18.0), t, F32x4::splat(-6.0/18.0))
}

#[inline(always)]
fn sampler_2x2(
    ctx: &super::SamplerCtx,
    cx: F32x4, cy: F32x4,
    wx: &[F32x4; 2], wy: &[F32x4; 2],
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
) {
    *r = F32x4::default();
    *g = F32x4::default();
    *b = F32x4::default();
    *a = F32x4::default();

    let one = F32x4::splat(1.0);
    let start = -0.5;
    let mut y = cy + F32x4::splat(start);
    for j in 0..2 {
        let mut x = cx + F32x4::splat(start);
        for i in 0..2 {
            let mut rr = F32x4::default();
            let mut gg = F32x4::default();
            let mut bb = F32x4::default();
            let mut aa = F32x4::default();
            sample(ctx, x,y, &mut rr, &mut gg, &mut bb, &mut aa);

            let w = wx[i] * wy[j];
            *r = mad(w, rr, *r);
            *g = mad(w, gg, *g);
            *b = mad(w, bb, *b);
            *a = mad(w, aa, *a);

            x = x + one;
        }

        y = y + one;
    }
}

#[inline(always)]
fn sampler_4x4(
    ctx: &super::SamplerCtx,
    cx: F32x4, cy: F32x4,
    wx: &[F32x4; 4], wy: &[F32x4; 4],
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
) {
    *r = F32x4::default();
    *g = F32x4::default();
    *b = F32x4::default();
    *a = F32x4::default();

    let one = F32x4::splat(1.0);
    let start = -1.5;
    let mut y = cy + F32x4::splat(start);
    for j in 0..4 {
        let mut x = cx + F32x4::splat(start);
        for i in 0..4 {
            let mut rr = F32x4::default();
            let mut gg = F32x4::default();
            let mut bb = F32x4::default();
            let mut aa = F32x4::default();
            sample(ctx, x,y, &mut rr, &mut gg, &mut bb, &mut aa);

            let w = wx[i] * wy[j];
            *r = mad(w, rr, *r);
            *g = mad(w, gg, *g);
            *b = mad(w, bb, *b);
            *a = mad(w, aa, *a);

            x = x + one;
        }

        y = y + one;
    }
}

#[inline(always)]
fn sample(
    ctx: &super::SamplerCtx, mut x: F32x4, mut y: F32x4,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
) {
    x = tile(x, ctx.spread_mode, ctx.gather.width.get() as f32, ctx.inv_width);
    y = tile(y, ctx.spread_mode, ctx.gather.height.get() as f32, ctx.inv_height);

    let ix = gather_ix(&ctx.gather, x, y);
    let data = unsafe { [
        *ctx.gather.pixels.cast::<PremultipliedColorU8>().add(ix.x() as usize),
        *ctx.gather.pixels.cast::<PremultipliedColorU8>().add(ix.y() as usize),
        *ctx.gather.pixels.cast::<PremultipliedColorU8>().add(ix.z() as usize),
        *ctx.gather.pixels.cast::<PremultipliedColorU8>().add(ix.w() as usize),
    ] };
    load_8888(&data, r, g, b, a);
}

#[inline(always)]
fn tile(v: F32x4, mode: SpreadMode, limit: f32, inv_limit: f32) -> F32x4 {
    // This match make this function almost 2x slower when building with `-Ctarget-cpu=haswell`.
    // TODO: optimize
    match mode {
        SpreadMode::Pad => v,
        SpreadMode::Repeat => exclusive_repeat(v, limit, inv_limit),
        SpreadMode::Reflect => exclusive_reflect(v, limit, inv_limit),
    }
}

fn pad_x1(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *r = r.normalize();

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn reflect_x1(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *r = (
        (*r - F32x4::splat(1.0))
            - two(((*r - F32x4::splat(1.0)) * F32x4::splat(0.5)).floor())
            - F32x4::splat(1.0)
    ).abs().normalize();

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn repeat_x1(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    *r = (*r - r.floor()).normalize();

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn gradient(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::GradientCtx = cast_stage(program);

    // N.B. The loop starts at 1 because idx 0 is the color to use before the first stop.
    let t = *r;
    let mut idx = U32x4::default();
    for i in 1..ctx.len {
        let tt = ctx.t_values[i].get();
        let n = U32x4::new(
            (t.x() >= tt) as u32,
            (t.y() >= tt) as u32,
            (t.z() >= tt) as u32,
            (t.w() >= tt) as u32,
        );
        idx = idx + n;
    }
    gradient_lookup(ctx, &idx, t, r, g, b, a);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn gradient_lookup(
    ctx: &super::GradientCtx, idx: &U32x4, t: F32x4,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
) {
    macro_rules! gather {
        ($d:expr, $c:ident) => {
            // Surprisingly, but bound checking doesn't affect the performance.
            // And since `idx` can contain any number, we should leave it in place.
            F32x4::new(
                $d[idx.x() as usize].$c,
                $d[idx.y() as usize].$c,
                $d[idx.z() as usize].$c,
                $d[idx.w() as usize].$c,
            )
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

    *r = mad(t, fr, br);
    *g = mad(t, fg, bg);
    *b = mad(t, fb, bb);
    *a = mad(t, fa, ba);
}

fn evenly_spaced_2_stop_gradient(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::EvenlySpaced2StopGradientCtx = cast_stage(program);

    let t = *r;
    *r = mad(t, F32x4::splat(ctx.factor.r), F32x4::splat(ctx.bias.r));
    *g = mad(t, F32x4::splat(ctx.factor.g), F32x4::splat(ctx.bias.g));
    *b = mad(t, F32x4::splat(ctx.factor.b), F32x4::splat(ctx.bias.b));
    *a = mad(t, F32x4::splat(ctx.factor.a), F32x4::splat(ctx.bias.a));

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn xy_to_radius(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let x2 = *r * *r;
    let y2 = *g * *g;
    *r = (x2 + y2).sqrt();

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn xy_to_2pt_conical_focal_on_circle(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let x = *r;
    let y = *g;
    *r = x + y * y / x;

    next_stage(tail, program, 1, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn xy_to_2pt_conical_well_behaved(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::TwoPointConicalGradientCtx = cast_stage(program);

    let x = *r;
    let y = *g;
    *r = (x * x + y * y).sqrt() - x * F32x4::splat(ctx.p0);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn xy_to_2pt_conical_greater(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::TwoPointConicalGradientCtx = cast_stage(program);

    let x = *r;
    let y = *g;
    *r = (x * x - y * y).sqrt() - x * F32x4::splat(ctx.p0);

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn mask_2pt_conical_degenerates(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &mut super::TwoPointConicalGradientCtx = cast_stage_mut(program);

    let t = *r;
    let is_degenerate = t.packed_le(F32x4::default()) | t.packed_ne(t);
    *r = is_degenerate.if_then_else(F32x4::default(), t);

    let is_not_degenerate = !is_degenerate;
    ctx.mask[0] = if is_not_degenerate.x() != 0 { !0 } else { 0 };
    ctx.mask[1] = if is_not_degenerate.y() != 0 { !0 } else { 0 };
    ctx.mask[2] = if is_not_degenerate.z() != 0 { !0 } else { 0 };
    ctx.mask[3] = if is_not_degenerate.w() != 0 { !0 } else { 0 };

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

fn apply_vector_mask(
    tail: usize, program: *const *const c_void, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    let ctx: &super::TwoPointConicalGradientCtx = cast_stage(program);

    let mask = U32x4::new(ctx.mask[0], ctx.mask[1], ctx.mask[2], ctx.mask[3]);
    *r = (r.to_u32x4_bitcast() & mask).to_f32x4_bitcast();
    *g = (g.to_u32x4_bitcast() & mask).to_f32x4_bitcast();
    *b = (b.to_u32x4_bitcast() & mask).to_f32x4_bitcast();
    *a = (a.to_u32x4_bitcast() & mask).to_f32x4_bitcast();

    next_stage(tail, program, 2, dx,dy, r,g,b,a, dr,dg,db,da);
}

pub fn just_return(
    _: usize, _: *const *const c_void, _: usize, _: usize,
    _: &mut F32x4, _: &mut F32x4, _: &mut F32x4, _: &mut F32x4,
    _: &mut F32x4, _: &mut F32x4, _: &mut F32x4, _: &mut F32x4,
) {
    // Ends the loop.
}

#[inline(always)]
fn cast_stage<T>(program: *const *const c_void) -> &'static T {
    unsafe { &*(*program.add(1)).cast() }
}

#[inline(always)]
fn cast_stage_mut<T>(program: *const *const c_void) -> &'static mut T {
    unsafe { &mut *(*program.add(1) as *mut c_void).cast() }
}

#[inline(always)]
fn next_stage(
    tail: usize, program: *const *const c_void, offset: usize, dx: usize, dy: usize,
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
    dr: &mut F32x4, dg: &mut F32x4, db: &mut F32x4, da: &mut F32x4,
) {
    unsafe {
        let next: StageFn = *program.add(offset).cast();
        next(tail, program.add(offset), dx,dy, r,g,b,a, dr,dg,db,da);
    }
}

#[inline(always)]
fn load_8888(
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
fn load_8888_tail(
    tail: usize, data: &[PremultipliedColorU8],
    r: &mut F32x4, g: &mut F32x4, b: &mut F32x4, a: &mut F32x4,
) {
    // Fill a dummy array with `tail` values. `tail` is always in a 1..STAGE_WIDTH-1 range.
    // This way we can reuse the `load_8888_` method and remove any branches.
    let mut tmp = [PremultipliedColorU8::TRANSPARENT; STAGE_WIDTH];
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), tmp.as_mut_ptr(), tail); }
    load_8888(&tmp, r, g, b, a);
}

#[inline(always)]
fn store_8888(
    r: &F32x4, g: &F32x4, b: &F32x4, a: &F32x4,
    data: &mut [PremultipliedColorU8; STAGE_WIDTH],
) {
    let r = unnorm(r);
    let g = unnorm(g);
    let b = unnorm(b);
    let a = unnorm(a);

    let conv = |rr, gg, bb, aa|
        PremultipliedColorU8::from_rgba_unchecked(rr as u8, gg as u8, bb as u8, aa as u8);

    data[0] = conv(r.x(), g.x(), b.x(), a.x());
    data[1] = conv(r.y(), g.y(), b.y(), a.y());
    data[2] = conv(r.z(), g.z(), b.z(), a.z());
    data[3] = conv(r.w(), g.w(), b.w(), a.w());
}

#[inline(always)]
fn store_8888_tail(
    r: &F32x4, g: &F32x4, b: &F32x4, a: &F32x4,
    tail: usize, data: &mut [PremultipliedColorU8],
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
        data[i] = PremultipliedColorU8::from_rgba_unchecked(
            r[i] as u8, g[i] as u8, b[i] as u8, a[i] as u8,
        );

        if i + 1 == tail {
            break;
        }
    }
}

#[inline(always)]
fn unnorm(v: &F32x4) -> I32x4 {
    (v.max(F32x4::default()).min(F32x4::splat(1.0)) * F32x4::splat(255.0)).to_i32x4_round()
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
