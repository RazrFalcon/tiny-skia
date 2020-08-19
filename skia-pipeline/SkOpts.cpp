/*
 * Copyright 2015 Google Inc.
 *
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#include "SkOpts.h"

#if defined(SK_ARM_HAS_NEON)
    #if defined(SK_ARM_HAS_CRC32)
        #define SK_OPTS_NS neon_and_crc32
    #else
        #define SK_OPTS_NS neon
    #endif
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SKX
    #define SK_OPTS_NS skx
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_AVX2
    #define SK_OPTS_NS avx2
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_AVX
    #define SK_OPTS_NS avx
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SSE42
    #define SK_OPTS_NS sse42
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SSE41
    #define SK_OPTS_NS sse41
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SSSE3
    #define SK_OPTS_NS ssse3
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SSE3
    #define SK_OPTS_NS sse3
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SSE2
    #define SK_OPTS_NS sse2
#elif SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SSE1
    #define SK_OPTS_NS sse
#else
    #define SK_OPTS_NS portable
#endif

#include "SkRasterPipeline_opts.h"

namespace SkOpts {
#define M(st) (StageFn)SK_OPTS_NS::st,
    StageFn stages_highp[] = { SK_RASTER_PIPELINE_STAGES(M) };
    StageFn just_return_highp = (StageFn)SK_OPTS_NS::just_return;
    void (*start_pipeline_highp)(size_t,size_t,size_t,size_t,void**)
        = SK_OPTS_NS::start_pipeline;
#undef M

#define M(st) (StageFn)SK_OPTS_NS::lowp::st,
    StageFn stages_lowp[] = { SK_RASTER_PIPELINE_STAGES(M) };
    StageFn just_return_lowp = (StageFn)SK_OPTS_NS::lowp::just_return;
    void (*start_pipeline_lowp)(size_t,size_t,size_t,size_t,void**)
        = SK_OPTS_NS::lowp::start_pipeline;
#undef M
}  // namespace SkOpts
