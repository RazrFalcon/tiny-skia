// Copyright 2018 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/*!
A high precision raster pipeline implementation.

Unlike lowp, this one implements all stages.

Just like Skia, this pipeline is implemented using f32x4.
Skia also supports F32x8 on modern CPUs, but we're not at the moment.

For some reason, we are almost 2x slower. Maybe because Skia uses clang's vector extensions
and we're using a manual implementation.
*/

use std::ffi::c_void;

use wide::{CmpEq, CmpLe, CmpGt, CmpGe, CmpNe};

use crate::{ScreenIntRect, PremultipliedColorU8, Transform, SpreadMode};

use crate::wide::{f32x4, i32x4, u32x4, F32x4Ext, I32x4Ext, U32x4Ext};
use crate::pipeline::BasePipeline;

pub const STAGE_WIDTH: usize = 4;

type StageFn = fn(p: &mut Pipeline);

pub struct Pipeline {
    pub program: *const *const c_void,
    pub r: f32x4,
    pub g: f32x4,
    pub b: f32x4,
    pub a: f32x4,
    pub dr: f32x4,
    pub dg: f32x4,
    pub db: f32x4,
    pub da: f32x4,
    pub tail: usize,
    pub dx: usize,
    pub dy: usize,
}

impl BasePipeline for Pipeline {
    #[inline(always)]
    fn program(&self) -> *const *const c_void {
        self.program
    }

    #[inline(always)]
    fn set_program(&mut self, p: *const *const c_void) {
        self.program = p;
    }
}


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
    let mut p = Pipeline {
        program: std::ptr::null(),
        r: f32x4::default(),
        g: f32x4::default(),
        b: f32x4::default(),
        a: f32x4::default(),
        dr: f32x4::default(),
        dg: f32x4::default(),
        db: f32x4::default(),
        da: f32x4::default(),
        tail: 0,
        dx: 0,
        dy: 0,
    };

    for y in rect.y()..rect.bottom() {
        let mut x = rect.x() as usize;
        let end = rect.right() as usize;

        while x + STAGE_WIDTH <= end {
            p.program = program;
            p.dx = x;
            p.dy = y as usize;
            p.tail = STAGE_WIDTH;
            p.next_stage(0);
            x += STAGE_WIDTH;
        }

        if x != end {
            p.program = tail_program;
            p.dx = x;
            p.dy = y as usize;
            p.tail = end - x;
            p.next_stage(0);
        }
    }
}

fn move_source_to_destination(p: &mut Pipeline) {
    p.dr = p.r;
    p.dg = p.g;
    p.db = p.b;
    p.da = p.a;

    p.next_stage(1);
}

fn premultiply(p: &mut Pipeline) {
    p.r *= p.a;
    p.g *= p.a;
    p.b *= p.a;

    p.next_stage(1);
}

fn move_destination_to_source(p: &mut Pipeline) {
    p.r = p.dr;
    p.g = p.dg;
    p.b = p.db;
    p.a = p.da;

    p.next_stage(1);
}

fn clamp_0(p: &mut Pipeline) {
    p.r = p.r.max(f32x4::default());
    p.g = p.g.max(f32x4::default());
    p.b = p.b.max(f32x4::default());
    p.a = p.a.max(f32x4::default());

    p.next_stage(1);
}

fn clamp_a(p: &mut Pipeline) {
    p.r = p.r.min(f32x4::splat(1.0));
    p.g = p.g.min(f32x4::splat(1.0));
    p.b = p.b.min(f32x4::splat(1.0));
    p.a = p.a.min(f32x4::splat(1.0));

    p.next_stage(1);
}

fn uniform_color(p: &mut Pipeline) {
    let ctx: &super::UniformColorCtx = p.stage_ctx();
    p.r = f32x4::splat(ctx.r);
    p.g = f32x4::splat(ctx.g);
    p.b = f32x4::splat(ctx.b);
    p.a = f32x4::splat(ctx.a);

    p.next_stage(2);
}

fn seed_shader(p: &mut Pipeline) {
    let iota = f32x4::from([0.5, 1.5, 2.5, 3.5]);

    p.r = f32x4::splat(p.dx as f32) + iota;
    p.g = f32x4::splat(p.dy as f32 + 0.5);
    p.b = f32x4::splat(1.0);
    p.a = f32x4::default();

    p.dr = f32x4::default();
    p.dg = f32x4::default();
    p.db = f32x4::default();
    p.da = f32x4::default();

    p.next_stage(1);
}

pub fn load_dst(p: &mut Pipeline) {
    let ctx: &mut super::PixelsCtx = p.stage_ctx_mut();
    load_8888(ctx.slice4_at_xy(p.dx, p.dy), &mut p.dr, &mut p.dg, &mut p.db, &mut p.da);

    p.next_stage(2);
}

pub fn load_dst_tail(p: &mut Pipeline) {
    let ctx: &mut super::PixelsCtx = p.stage_ctx_mut();
    load_8888_tail(p.tail, ctx.slice_at_xy(p.dx, p.dy), &mut p.dr, &mut p.dg, &mut p.db, &mut p.da);

    p.next_stage(2);
}

pub fn store(p: &mut Pipeline) {
    let ctx: &mut super::PixelsCtx = p.stage_ctx_mut();
    store_8888(&p.r, &p.g, &p.b, &p.a, ctx.slice4_at_xy(p.dx, p.dy));

    p.next_stage(2);
}

pub fn gather(p: &mut Pipeline) {
    let ctx: &super::GatherCtx = p.stage_ctx();

    let ix = gather_ix(ctx, p.r, p.g);
    load_8888(&ctx.gather(ix), &mut p.r, &mut p.g, &mut p.b, &mut p.a);

    p.next_stage(2);
}

#[inline(always)]
fn gather_ix(ctx: &super::GatherCtx, mut x: f32x4, mut y: f32x4) -> u32x4 {
    // Exclusive -> inclusive.
    let w = ulp_sub(ctx.width.get() as f32);
    let h = ulp_sub(ctx.height.get() as f32);
    x = x.max(f32x4::default()).min(f32x4::splat(w));
    y = y.max(f32x4::default()).min(f32x4::splat(h));

    (y.trunc_int() * i32x4::splat(ctx.stride.get() as i32) + x.trunc_int()).to_u32x4_bitcast()
}

#[inline(always)]
fn ulp_sub(v: f32) -> f32 {
    // Somewhat similar to v - f32::EPSILON
    bytemuck::cast::<u32, f32>(bytemuck::cast::<f32, u32>(v) - 1)
}

pub fn store_tail(p: &mut Pipeline) {
    let ctx: &mut super::PixelsCtx = p.stage_ctx_mut();
    store_8888_tail(&p.r, &p.g, &p.b, &p.a, p.tail, ctx.slice_at_xy(p.dx, p.dy));

    p.next_stage(2);
}

fn scale_u8(p: &mut Pipeline) {
    let ctx: &super::MaskCtx = p.stage_ctx();

    // Load u8xTail and cast it to f32x4.
    let data = ctx.copy_at_xy(p.dx, p.dy, p.tail);
    let c = f32x4::from([data[0] as f32, data[1] as f32, 0.0, 0.0]);
    let c = c / f32x4::splat(255.0);

    p.r *= c;
    p.g *= c;
    p.b *= c;
    p.a *= c;

    p.next_stage(2);
}

fn lerp_u8(p: &mut Pipeline) {
    let ctx: &super::MaskCtx = p.stage_ctx();

    // Load u8xTail and cast it to f32x4.
    let data = ctx.copy_at_xy(p.dx, p.dy, p.tail);
    let c = f32x4::from([data[0] as f32, data[1] as f32, 0.0, 0.0]);
    let c = c / f32x4::splat(255.0);

    p.r = lerp(p.dr, p.r, c);
    p.g = lerp(p.dg, p.g, c);
    p.b = lerp(p.db, p.b, c);
    p.a = lerp(p.da, p.a, c);

    p.next_stage(2);
}

fn scale_1_float(p: &mut Pipeline) {
    let c: f32 = *p.stage_ctx();
    let c = f32x4::splat(c);
    p.r *= c;
    p.g *= c;
    p.b *= c;
    p.a *= c;

    p.next_stage(2);
}

fn lerp_1_float(p: &mut Pipeline) {
    let c: f32 = *p.stage_ctx();
    let c = f32x4::splat(c);
    p.r = lerp(p.dr, p.r, c);
    p.g = lerp(p.dg, p.g, c);
    p.b = lerp(p.db, p.b, c);
    p.a = lerp(p.da, p.a, c);

    p.next_stage(2);
}

macro_rules! blend_fn {
    ($name:ident, $f:expr) => {
        fn $name(p: &mut Pipeline) {
            p.r = $f(p.r, p.dr, p.a, p.da);
            p.g = $f(p.g, p.dg, p.a, p.da);
            p.b = $f(p.b, p.db, p.a, p.da);
            p.a = $f(p.a, p.da, p.a, p.da);

            p.next_stage(1);
        }
    };
}

blend_fn!(clear,            |_, _,  _,  _| f32x4::default());
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
blend_fn!(plus, |s: f32x4, d: f32x4, _, _| (s + d).min(f32x4::splat(1.0)));

macro_rules! blend_fn2 {
    ($name:ident, $f:expr) => {
        fn $name(p: &mut Pipeline) {
            // The same logic applied to color, and source_over for alpha.
            p.r = $f(p.r, p.dr, p.a, p.da);
            p.g = $f(p.g, p.dg, p.a, p.da);
            p.b = $f(p.b, p.db, p.a, p.da);
            p.a = mad(p.da, inv(p.a), p.a);

            p.next_stage(1);
        }
    };
}

blend_fn2!(darken,      |s: f32x4, d, sa, da: f32x4| s + d - (s * da).max(d * sa));
blend_fn2!(lighten,     |s: f32x4, d, sa, da: f32x4| s + d - (s * da).min(d * sa));
blend_fn2!(difference,  |s: f32x4, d, sa, da: f32x4| s + d - two((s * da).min(d * sa)));
blend_fn2!(exclusion,   |s: f32x4, d,  _,  _| s + d - two(s * d));

blend_fn2!(color_burn, |s: f32x4, d: f32x4, sa: f32x4, da: f32x4|
    d.cmp_eq(da).blend(
        d + s * inv(da),
        s.cmp_eq(f32x4::default()).blend(
            d * inv(sa),
            sa * (da - da.min((da - d) * sa * s.recip())) + s * inv(da) + d * inv(sa)
        )
    )
);

blend_fn2!(color_dodge, |s: f32x4, d: f32x4, sa: f32x4, da: f32x4|
    d.cmp_eq(f32x4::default()).blend(
        s * inv(da),
        s.cmp_eq(sa).blend(
            s + d * inv(sa),
            sa * da.min((d * sa) * (sa - s).recip()) + s * inv(da) + d * inv(sa)
        )
    )
);

blend_fn2!(hard_light, |s: f32x4, d: f32x4, sa, da|
    s * inv(da) + d * inv(sa) + two(s).cmp_le(sa).blend(
        two(s * d),
        sa * da - two((da - d) * (sa - s))
    )
);

blend_fn2!(overlay, |s: f32x4, d: f32x4, sa, da|
    s * inv(da) + d * inv(sa) + two(d).cmp_le(da).blend(
        two(s * d),
        sa * da - two((da - d) * (sa - s))
    )
);

blend_fn2!(soft_light, |s: f32x4, d: f32x4, sa: f32x4, da: f32x4| {
    let m  = da.cmp_gt(f32x4::default()).blend(d / da, f32x4::default());
    let s2 = two(s);
    let m4 = two(two(m));

    // The logic forks three ways:
    //    1. dark src?
    //    2. light src, dark dst?
    //    3. light src, light dst?
    let dark_src = d * (sa + (s2 - sa) * (f32x4::splat(1.0) - m));
    let dark_dst = (m4 * m4 + m4) * (m - f32x4::splat(1.0)) + f32x4::splat(7.0) * m;
    let lite_dst = m.recip_sqrt().recip() - m;
    let lite_src = d * sa + da * (s2 - sa)
        * two(two(d)).cmp_le(da).blend(dark_dst, lite_dst); // 2 or 3?

    s*inv(da) + d*inv(sa) + s2.cmp_le(sa).blend(dark_src, lite_src) // 1 or (2 or 3)?
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
        fn $name(p: &mut Pipeline) {
            let (tr, tg, tb, ta) = $f(p.r, p.g, p.b, p.a, p.dr, p.dg, p.db, p.da);
            p.r = tr;
            p.g = tg;
            p.b = tb;
            p.a = ta;

            p.next_stage(1);
        }
    };
}

blend_fn3!(hue, hue_k);

#[inline(always)]
fn hue_k(
    r: f32x4, g: f32x4, b: f32x4, a: f32x4,
    dr: f32x4, dg: f32x4, db: f32x4, da: f32x4,
) -> (f32x4, f32x4, f32x4, f32x4) {
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
    r: f32x4, g: f32x4, b: f32x4, a: f32x4,
    dr: f32x4, dg: f32x4, db: f32x4, da: f32x4,
) -> (f32x4, f32x4, f32x4, f32x4) {
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
    r: f32x4, g: f32x4, b: f32x4, a: f32x4,
    dr: f32x4, dg: f32x4, db: f32x4, da: f32x4,
) -> (f32x4, f32x4, f32x4, f32x4) {
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
    r: f32x4, g: f32x4, b: f32x4, a: f32x4,
    dr: f32x4, dg: f32x4, db: f32x4, da: f32x4,
) -> (f32x4, f32x4, f32x4, f32x4) {
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
fn sat(r: f32x4, g: f32x4, b: f32x4) -> f32x4 {
    r.max(g.max(b)) - r.min(g.min(b))
}

#[inline(always)]
fn lum(r: f32x4, g: f32x4, b: f32x4) -> f32x4 {
    r * f32x4::splat(0.30) + g * f32x4::splat(0.59) + b * f32x4::splat(0.11)
}

#[inline(always)]
fn set_sat(r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, s: f32x4) {
    let mn  = r.min(g.min(*b));
    let mx  = r.max(g.max(*b));
    let sat = mx - mn;

    // Map min channel to 0, max channel to s, and scale the middle proportionally.
    let scale = |c| sat.cmp_eq(f32x4::default())
                       .blend(f32x4::default(), (c - mn) * s / sat);

    *r = scale(*r);
    *g = scale(*g);
    *b = scale(*b);
}

#[inline(always)]
fn set_lum(r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, l: f32x4) {
    let diff = l - lum(*r, *g, *b);
    *r += diff;
    *g += diff;
    *b += diff;
}

#[inline(always)]
fn clip_color(r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, a: f32x4) {
    let mn = r.min(g.min(*b));
    let mx = r.max(g.max(*b));
    let l  = lum(*r, *g, *b);

    let clip = |mut c| {
        c = mx.cmp_ge(f32x4::default()).blend(c, l + (c - l) * l / (l - mn));
        c = mx.cmp_gt(a).blend(l + (c - l) * (a - l) / (mx - l), c);
        c = c.max(f32x4::default()); // Sometimes without this we may dip just a little negative.
        c
    };

    *r = clip(*r);
    *g = clip(*g);
    *b = clip(*b);
}

pub fn source_over_rgba(p: &mut Pipeline) {
    let ctx: &mut super::PixelsCtx = p.stage_ctx_mut();
    let pixels = ctx.slice4_at_xy(p.dx, p.dy);
    load_8888(pixels, &mut p.dr, &mut p.dg, &mut p.db, &mut p.da);
    p.r = mad(p.dr, inv(p.a), p.r);
    p.g = mad(p.dg, inv(p.a), p.g);
    p.b = mad(p.db, inv(p.a), p.b);
    p.a = mad(p.da, inv(p.a), p.a);
    store_8888(&p.r, &p.g, &p.b, &p.a, pixels);

    p.next_stage(2);
}

pub fn source_over_rgba_tail(p: &mut Pipeline) {
    let ctx: &mut super::PixelsCtx = p.stage_ctx_mut();
    let pixels = ctx.slice_at_xy(p.dx, p.dy);
    load_8888_tail(p.tail, pixels, &mut p.dr, &mut p.dg, &mut p.db, &mut p.da);
    p.r = mad(p.dr, inv(p.a), p.r);
    p.g = mad(p.dg, inv(p.a), p.g);
    p.b = mad(p.db, inv(p.a), p.b);
    p.a = mad(p.da, inv(p.a), p.a);
    store_8888_tail(&p.r, &p.g, &p.b, &p.a, p.tail, pixels);

    p.next_stage(2);
}

fn transform(p: &mut Pipeline) {
    let ts: &Transform = p.stage_ctx();
    let (sx, ky, kx, sy, tx, ty) = ts.get_row();

    let tr = mad(p.r, f32x4::splat(sx), mad(p.g, f32x4::splat(kx), f32x4::splat(tx)));
    let tg = mad(p.r, f32x4::splat(ky), mad(p.g, f32x4::splat(sy), f32x4::splat(ty)));
    p.r = tr;
    p.g = tg;

    p.next_stage(2);
}

// Tile x or y to [0,limit) == [0,limit - 1 ulp] (think, sampling from images).
// The gather stages will hard clamp the output of these stages to [0,limit)...
// we just need to do the basic repeat or mirroring.

fn reflect_x(p: &mut Pipeline) {
    let ctx: &super::TileCtx = p.stage_ctx();
    p.r = exclusive_reflect(p.r, ctx.scale, ctx.inv_scale);

    p.next_stage(2);
}

fn reflect_y(p: &mut Pipeline) {
    let ctx: &super::TileCtx = p.stage_ctx();
    p.g = exclusive_reflect(p.g, ctx.scale, ctx.inv_scale);

    p.next_stage(2);
}

#[inline(always)]
fn exclusive_reflect(v: f32x4, limit: f32, inv_limit: f32) -> f32x4 {
    let limit = f32x4::splat(limit);
    let inv_limit = f32x4::splat(inv_limit);
    ((v - limit) - (limit + limit) * ((v - limit) * (inv_limit * f32x4::splat(0.5))).floor() - limit).abs()
}

fn repeat_x(p: &mut Pipeline) {
    let ctx: &super::TileCtx = p.stage_ctx();
    p.r = exclusive_repeat(p.r, ctx.scale, ctx.inv_scale);

    p.next_stage(2);
}

fn repeat_y(p: &mut Pipeline) {
    let ctx: &super::TileCtx = p.stage_ctx();
    p.g = exclusive_repeat(p.g, ctx.scale, ctx.inv_scale);

    p.next_stage(2);
}

#[inline(always)]
fn exclusive_repeat(v: f32x4, limit: f32, inv_limit: f32) -> f32x4 {
    v - (v * f32x4::splat(inv_limit)).floor() * f32x4::splat(limit)
}

fn bilinear(p: &mut Pipeline) {
    let ctx: &super::SamplerCtx = p.stage_ctx();

    let x = p.r;
    let fx = (x + f32x4::splat(0.5)).fract();
    let y = p.g;
    let fy = (y + f32x4::splat(0.5)).fract();
    let one = f32x4::splat(1.0);
    let wx = [one - fx, fx];
    let wy = [one - fy, fy];

    sampler_2x2(ctx, x, y, &wx, &wy, &mut p.r, &mut p.g, &mut p.b, &mut p.a);

    p.next_stage(2);
}

fn bicubic(p: &mut Pipeline) {
    let ctx: &super::SamplerCtx = p.stage_ctx();

    let x = p.r;
    let fx = (x + f32x4::splat(0.5)).fract();
    let y = p.g;
    let fy = (y + f32x4::splat(0.5)).fract();
    let one = f32x4::splat(1.0);
    let wx = [bicubic_far(one - fx), bicubic_near(one - fx), bicubic_near(fx), bicubic_far(fx)];
    let wy = [bicubic_far(one - fy), bicubic_near(one - fy), bicubic_near(fy), bicubic_far(fy)];

    sampler_4x4(ctx, x, y, &wx, &wy, &mut p.r, &mut p.g, &mut p.b, &mut p.a);

    p.next_stage(2);
}

// In bicubic interpolation, the 16 pixels and +/- 0.5 and +/- 1.5 offsets from the sample
// pixel center are combined with a non-uniform cubic filter, with higher values near the center.
//
// We break this function into two parts, one for near 0.5 offsets and one for far 1.5 offsets.

#[inline(always)]
fn bicubic_near(t: f32x4) -> f32x4 {
    // 1/18 + 9/18t + 27/18t^2 - 21/18t^3 == t ( t ( -21/18t + 27/18) + 9/18) + 1/18
    mad(t, mad(t, mad(f32x4::splat(-21.0/18.0), t, f32x4::splat(27.0/18.0)), f32x4::splat(9.0/18.0)), f32x4::splat(1.0/18.0))
}

#[inline(always)]
fn bicubic_far(t: f32x4) -> f32x4 {
    // 0/18 + 0/18*t - 6/18t^2 + 7/18t^3 == t^2 (7/18t - 6/18)
    (t * t) * mad(f32x4::splat(7.0/18.0), t, f32x4::splat(-6.0/18.0))
}

#[inline(always)]
fn sampler_2x2(
    ctx: &super::SamplerCtx,
    cx: f32x4, cy: f32x4,
    wx: &[f32x4; 2], wy: &[f32x4; 2],
    r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, a: &mut f32x4,
) {
    *r = f32x4::default();
    *g = f32x4::default();
    *b = f32x4::default();
    *a = f32x4::default();

    let one = f32x4::splat(1.0);
    let start = -0.5;
    let mut y = cy + f32x4::splat(start);
    for j in 0..2 {
        let mut x = cx + f32x4::splat(start);
        for i in 0..2 {
            let mut rr = f32x4::default();
            let mut gg = f32x4::default();
            let mut bb = f32x4::default();
            let mut aa = f32x4::default();
            sample(ctx, x,y, &mut rr, &mut gg, &mut bb, &mut aa);

            let w = wx[i] * wy[j];
            *r = mad(w, rr, *r);
            *g = mad(w, gg, *g);
            *b = mad(w, bb, *b);
            *a = mad(w, aa, *a);

            x += one;
        }

        y += one;
    }
}

#[inline(always)]
fn sampler_4x4(
    ctx: &super::SamplerCtx,
    cx: f32x4, cy: f32x4,
    wx: &[f32x4; 4], wy: &[f32x4; 4],
    r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, a: &mut f32x4,
) {
    *r = f32x4::default();
    *g = f32x4::default();
    *b = f32x4::default();
    *a = f32x4::default();

    let one = f32x4::splat(1.0);
    let start = -1.5;
    let mut y = cy + f32x4::splat(start);
    for j in 0..4 {
        let mut x = cx + f32x4::splat(start);
        for i in 0..4 {
            let mut rr = f32x4::default();
            let mut gg = f32x4::default();
            let mut bb = f32x4::default();
            let mut aa = f32x4::default();
            sample(ctx, x,y, &mut rr, &mut gg, &mut bb, &mut aa);

            let w = wx[i] * wy[j];
            *r = mad(w, rr, *r);
            *g = mad(w, gg, *g);
            *b = mad(w, bb, *b);
            *a = mad(w, aa, *a);

            x += one;
        }

        y += one;
    }
}

#[inline(always)]
fn sample(
    ctx: &super::SamplerCtx, mut x: f32x4, mut y: f32x4,
    r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, a: &mut f32x4,
) {
    x = tile(x, ctx.spread_mode, ctx.gather.width.get() as f32, ctx.inv_width);
    y = tile(y, ctx.spread_mode, ctx.gather.height.get() as f32, ctx.inv_height);

    let ix = gather_ix(&ctx.gather, x, y);
    load_8888(&ctx.gather.gather(ix), r, g, b, a);
}

#[inline(always)]
fn tile(v: f32x4, mode: SpreadMode, limit: f32, inv_limit: f32) -> f32x4 {
    // This match make this function almost 2x slower when building with `-Ctarget-cpu=haswell`.
    // TODO: optimize
    match mode {
        SpreadMode::Pad => v,
        SpreadMode::Repeat => exclusive_repeat(v, limit, inv_limit),
        SpreadMode::Reflect => exclusive_reflect(v, limit, inv_limit),
    }
}

fn pad_x1(p: &mut Pipeline) {
    p.r = p.r.normalize();

    p.next_stage(1);
}

fn reflect_x1(p: &mut Pipeline) {
    p.r = (
        (p.r - f32x4::splat(1.0))
            - two(((p.r - f32x4::splat(1.0)) * f32x4::splat(0.5)).floor())
            - f32x4::splat(1.0)
    ).abs().normalize();

    p.next_stage(1);
}

fn repeat_x1(p: &mut Pipeline) {
    p.r = (p.r - p.r.floor()).normalize();

    p.next_stage(1);
}

fn gradient(p: &mut Pipeline) {
    let ctx: &super::GradientCtx = p.stage_ctx();

    // N.B. The loop starts at 1 because idx 0 is the color to use before the first stop.
    let t: [f32; 4] = p.r.into();
    let mut idx = u32x4::default();
    for i in 1..ctx.len {
        let tt = ctx.t_values[i].get();
        let n = u32x4::from([
            (t[0] >= tt) as u32,
            (t[1] >= tt) as u32,
            (t[2] >= tt) as u32,
            (t[3] >= tt) as u32,
        ]);
        idx += n;
    }
    gradient_lookup(ctx, &idx, p.r, &mut p.r, &mut p.g, &mut p.b, &mut p.a);

    p.next_stage(2);
}

fn gradient_lookup(
    ctx: &super::GradientCtx, idx: &u32x4, t: f32x4,
    r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, a: &mut f32x4,
) {
    let idx: [u32; 4] = (*idx).into();

    macro_rules! gather {
        ($d:expr, $c:ident) => {
            // Surprisingly, but bound checking doesn't affect the performance.
            // And since `idx` can contain any number, we should leave it in place.
            f32x4::from([
                $d[idx[0] as usize].$c,
                $d[idx[1] as usize].$c,
                $d[idx[2] as usize].$c,
                $d[idx[3] as usize].$c,
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

    *r = mad(t, fr, br);
    *g = mad(t, fg, bg);
    *b = mad(t, fb, bb);
    *a = mad(t, fa, ba);
}

fn evenly_spaced_2_stop_gradient(p: &mut Pipeline) {
    let ctx: &super::EvenlySpaced2StopGradientCtx = p.stage_ctx();

    let t = p.r;
    p.r = mad(t, f32x4::splat(ctx.factor.r), f32x4::splat(ctx.bias.r));
    p.g = mad(t, f32x4::splat(ctx.factor.g), f32x4::splat(ctx.bias.g));
    p.b = mad(t, f32x4::splat(ctx.factor.b), f32x4::splat(ctx.bias.b));
    p.a = mad(t, f32x4::splat(ctx.factor.a), f32x4::splat(ctx.bias.a));

    p.next_stage(2);
}

fn xy_to_radius(p: &mut Pipeline) {
    let x2 = p.r * p.r;
    let y2 = p.g * p.g;
    p.r = (x2 + y2).sqrt();

    p.next_stage(1);
}

fn xy_to_2pt_conical_focal_on_circle(p: &mut Pipeline) {
    let x = p.r;
    let y = p.g;
    p.r = x + y * y / x;

    p.next_stage(1);
}

fn xy_to_2pt_conical_well_behaved(p: &mut Pipeline) {
    let ctx: &super::TwoPointConicalGradientCtx = p.stage_ctx();

    let x = p.r;
    let y = p.g;
    p.r = (x * x + y * y).sqrt() - x * f32x4::splat(ctx.p0);

    p.next_stage(2);
}

fn xy_to_2pt_conical_greater(p: &mut Pipeline) {
    let ctx: &super::TwoPointConicalGradientCtx = p.stage_ctx();

    let x = p.r;
    let y = p.g;
    p.r = (x * x - y * y).sqrt() - x * f32x4::splat(ctx.p0);

    p.next_stage(2);
}

fn mask_2pt_conical_degenerates(p: &mut Pipeline) {
    let ctx: &mut super::TwoPointConicalGradientCtx = p.stage_ctx_mut();

    let t = p.r;
    let is_degenerate = t.cmp_le(f32x4::default()) | t.cmp_ne(t);
    p.r = is_degenerate.blend(f32x4::default(), t);

    let is_not_degenerate = !is_degenerate.to_u32x4_bitcast();
    let is_not_degenerate: [u32; 4] = is_not_degenerate.into();
    ctx.mask = u32x4::from([
        if is_not_degenerate[0] != 0 { !0 } else { 0 },
        if is_not_degenerate[1] != 0 { !0 } else { 0 },
        if is_not_degenerate[2] != 0 { !0 } else { 0 },
        if is_not_degenerate[3] != 0 { !0 } else { 0 },
    ]);

    p.next_stage(2);
}

fn apply_vector_mask(p: &mut Pipeline) {
    let ctx: &super::TwoPointConicalGradientCtx = p.stage_ctx();

    p.r = (p.r.to_u32x4_bitcast() & ctx.mask).to_f32x4_bitcast();
    p.g = (p.g.to_u32x4_bitcast() & ctx.mask).to_f32x4_bitcast();
    p.b = (p.b.to_u32x4_bitcast() & ctx.mask).to_f32x4_bitcast();
    p.a = (p.a.to_u32x4_bitcast() & ctx.mask).to_f32x4_bitcast();

    p.next_stage(2);
}

pub fn just_return(
    _: usize, _: *const *const c_void, _: usize, _: usize,
    _: &mut f32x4, _: &mut f32x4, _: &mut f32x4, _: &mut f32x4,
    _: &mut f32x4, _: &mut f32x4, _: &mut f32x4, _: &mut f32x4,
) {
    // Ends the loop.
}

#[inline(always)]
fn load_8888(
    data: &[PremultipliedColorU8; STAGE_WIDTH],
    r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, a: &mut f32x4,
) {
    // Surprisingly, `f32 * FACTOR` is way faster than `f32x4 * f32x4::splat(FACTOR)`.

    const FACTOR: f32 = 1.0 / 255.0;

    *r = f32x4::from([
        data[0].red() as f32 * FACTOR, data[1].red() as f32 * FACTOR,
        data[2].red() as f32 * FACTOR, data[3].red() as f32 * FACTOR,
    ]);

    *g = f32x4::from([
        data[0].green() as f32 * FACTOR, data[1].green() as f32 * FACTOR,
        data[2].green() as f32 * FACTOR, data[3].green() as f32 * FACTOR,
    ]);

    *b = f32x4::from([
        data[0].blue() as f32 * FACTOR, data[1].blue() as f32 * FACTOR,
        data[2].blue() as f32 * FACTOR, data[3].blue() as f32 * FACTOR,
    ]);

    *a = f32x4::from([
        data[0].alpha() as f32 * FACTOR, data[1].alpha() as f32 * FACTOR,
        data[2].alpha() as f32 * FACTOR, data[3].alpha() as f32 * FACTOR,
    ]);
}

#[inline(always)]
fn load_8888_tail(
    tail: usize, data: &[PremultipliedColorU8],
    r: &mut f32x4, g: &mut f32x4, b: &mut f32x4, a: &mut f32x4,
) {
    // Fill a dummy array with `tail` values. `tail` is always in a 1..STAGE_WIDTH-1 range.
    // This way we can reuse the `load_8888_` method and remove any branches.
    let mut tmp = [PremultipliedColorU8::TRANSPARENT; STAGE_WIDTH];
    tmp[0..tail].copy_from_slice(&data[0..tail]);
    load_8888(&tmp, r, g, b, a);
}

#[inline(always)]
fn store_8888(
    r: &f32x4, g: &f32x4, b: &f32x4, a: &f32x4,
    data: &mut [PremultipliedColorU8; STAGE_WIDTH],
) {
    let r: [i32; 4] = bytemuck::cast(unnorm(r));
    let g: [i32; 4] = bytemuck::cast(unnorm(g));
    let b: [i32; 4] = bytemuck::cast(unnorm(b));
    let a: [i32; 4] = bytemuck::cast(unnorm(a));

    let conv = |rr, gg, bb, aa|
        PremultipliedColorU8::from_rgba_unchecked(rr as u8, gg as u8, bb as u8, aa as u8);

    data[0] = conv(r[0], g[0], b[0], a[0]);
    data[1] = conv(r[1], g[1], b[1], a[1]);
    data[2] = conv(r[2], g[2], b[2], a[2]);
    data[3] = conv(r[3], g[3], b[3], a[3]);
}

#[inline(always)]
fn store_8888_tail(
    r: &f32x4, g: &f32x4, b: &f32x4, a: &f32x4,
    tail: usize, data: &mut [PremultipliedColorU8],
) {
    let r: [i32; 4] = bytemuck::cast(unnorm(r));
    let g: [i32; 4] = bytemuck::cast(unnorm(g));
    let b: [i32; 4] = bytemuck::cast(unnorm(b));
    let a: [i32; 4] = bytemuck::cast(unnorm(a));

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
fn unnorm(v: &f32x4) -> i32x4 {
    (v.max(f32x4::default()).min(f32x4::splat(1.0)) * f32x4::splat(255.0)).round_int()
}

#[inline(always)]
fn inv(v: f32x4) -> f32x4 {
    f32x4::splat(1.0) - v
}

#[inline(always)]
fn two(v: f32x4) -> f32x4 {
    v + v
}

#[inline(always)]
fn mad(f: f32x4, m: f32x4, a: f32x4) -> f32x4 {
    f * m + a
}

#[inline(always)]
fn lerp(from: f32x4, to: f32x4, t: f32x4) -> f32x4 {
    mad(to - from, t, from)
}
