/*
 * Copyright 2016 Google Inc.
 *
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#ifndef SkRasterPipeline_DEFINED
#define SkRasterPipeline_DEFINED

#include "SkTypes.h"

/**
 * SkRasterPipeline provides a cheap way to chain together a pixel processing pipeline.
 *
 * It's particularly designed for situations where the potential pipeline is extremely
 * combinatoric: {N dst formats} x {M source formats} x {K mask formats} x {C transfer modes} ...
 * No one wants to write specialized routines for all those combinations, and if we did, we'd
 * end up bloating our code size dramatically.  SkRasterPipeline stages can be chained together
 * at runtime, so we can scale this problem linearly rather than combinatorically.
 *
 * Each stage is represented by a function conforming to a common interface and by an
 * arbitrary context pointer.  The stage funciton arguments and calling convention are
 * designed to maximize the amount of data we can pass along the pipeline cheaply, and
 * vary depending on CPU feature detection.
 */

#define SK_RASTER_PIPELINE_STAGES(M)                               \
    M(move_src_dst) M(move_dst_src)                                \
    M(clamp_0) M(clamp_1) M(clamp_a) M(clamp_gamut)                \
    M(unpremul) M(premul) M(premul_dst)                            \
    M(black_color) M(white_color)                                  \
    M(uniform_color) M(unbounded_uniform_color) M(uniform_color_dst) \
    M(seed_shader) M(dither)                                       \
    M(load_8888)   M(load_8888_dst) M(store_8888)  M(gather_8888)  \
    M(bilerp_clamp_8888) M(bicubic_clamp_8888)                     \
    M(load_src) M(store_src) M(store_src_a) M(load_dst) M(store_dst) \
    M(scale_u8) M(scale_1_float) M(scale_native)      \
    M( lerp_u8) M( lerp_1_float) M(lerp_native)       \
    M(dstatop) M(dstin) M(dstout) M(dstover)                       \
    M(srcatop) M(srcin) M(srcout) M(srcover)                       \
    M(clear) M(modulate) M(multiply) M(plus_) M(screen) M(xor_)    \
    M(colorburn) M(colordodge) M(darken) M(difference)             \
    M(exclusion) M(hardlight) M(lighten) M(overlay) M(softlight)   \
    M(hue) M(saturation) M(color) M(luminosity)                    \
    M(srcover_rgba_8888)                                           \
    M(matrix_translate) M(matrix_scale_translate)                  \
    M(matrix_2x3) M(matrix_3x3) M(matrix_3x4) M(matrix_4x5) M(matrix_4x3) \
    M(mirror_x)   M(repeat_x)                                      \
    M(mirror_y)   M(repeat_y)                                      \
    M(negate_x)                                                    \
    M(bilinear) M(bicubic)                                         \
    M(bilinear_nx) M(bilinear_px) M(bilinear_ny) M(bilinear_py)    \
    M(bicubic_n3x) M(bicubic_n1x) M(bicubic_p1x) M(bicubic_p3x)    \
    M(bicubic_n3y) M(bicubic_n1y) M(bicubic_p1y) M(bicubic_p3y)    \
    M(save_xy) M(accumulate)                                       \
    M(clamp_x_1) M(mirror_x_1) M(repeat_x_1)                       \
    M(evenly_spaced_gradient)                                      \
    M(gradient)                                                    \
    M(evenly_spaced_2_stop_gradient)                               \
    M(xy_to_unit_angle)                                            \
    M(xy_to_radius)                                                \
    M(xy_to_2pt_conical_strip)                                     \
    M(xy_to_2pt_conical_focal_on_circle)                           \
    M(xy_to_2pt_conical_well_behaved)                              \
    M(xy_to_2pt_conical_smaller)                                   \
    M(xy_to_2pt_conical_greater)                                   \
    M(alter_2pt_conical_compensate_focal)                          \
    M(alter_2pt_conical_unswap)                                    \
    M(mask_2pt_conical_nan)                                        \
    M(mask_2pt_conical_degenerates) M(apply_vector_mask)


// The largest number of pixels we handle at a time.
static const int SkRasterPipeline_kMaxStride = 16;

// Structs representing the arguments to some common stages.

struct SkRasterPipeline_MemoryCtx {
    void* pixels;
    int   stride;
};

struct SkRasterPipeline_GatherCtx {
    const void* pixels;
    int         stride;
    float       width;
    float       height;
};

// State shared by save_xy, accumulate, and bilinear_* / bicubic_*.
struct SkRasterPipeline_SamplerCtx {
    float      x[SkRasterPipeline_kMaxStride];
    float      y[SkRasterPipeline_kMaxStride];
    float     fx[SkRasterPipeline_kMaxStride];
    float     fy[SkRasterPipeline_kMaxStride];
    float scalex[SkRasterPipeline_kMaxStride];
    float scaley[SkRasterPipeline_kMaxStride];
};

struct SkRasterPipeline_TileCtx {
    float scale;
    float invScale; // cache of 1/scale
};

enum class SkTileMode {
    /**
     *  Replicate the edge color if the shader draws outside of its
     *  original bounds.
     */
    kClamp,

    /**
     *  Repeat the shader's image horizontally and vertically.
     */
    kRepeat,

    /**
     *  Repeat the shader's image horizontally and vertically, alternating
     *  mirror images so that adjacent images always seam.
     */
    kMirror,

    kLastTileMode = kMirror,
};

struct SkRasterPipeline_SamplerCtx2 : public SkRasterPipeline_GatherCtx {
    SkTileMode tileX, tileY;
    float invWidth, invHeight;
};

struct SkRasterPipeline_GradientCtx {
    size_t stopCount;
    float* fs[4];
    float* bs[4];
    float* ts;
};

struct SkRasterPipeline_EvenlySpaced2StopGradientCtx {
    float f[4];
    float b[4];
};

struct SkRasterPipeline_2PtConicalCtx {
    uint32_t fMask[SkRasterPipeline_kMaxStride];
    float    fP0,
             fP1;
};

struct SkRasterPipeline_UniformColorCtx {
    float r,g,b,a;
    uint16_t rgba[4];  // [0,255] in a 16-bit lane.
};

enum StockStage {
#define M(stage) stage,
    SK_RASTER_PIPELINE_STAGES(M)
#undef M
};

struct StageList {
    StageList* prev;
    StockStage stage;
    void*      ctx;
};

using StartPipelineFn = void(*)(size_t,size_t,size_t,size_t, void** program);

StartPipelineFn build_pipeline(StageList* stages, void** ip);

extern "C" {
bool skia_pipe_raster_build_pipeline(StageList *stages, void** ip);
void skia_pipe_raster_run_pipeline(void** program, bool is_highp, size_t x, size_t y, size_t w, size_t h);
}

#endif //SkRasterPipeline_DEFINED
