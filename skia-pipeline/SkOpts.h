/*
 * Copyright 2015 Google Inc.
 *
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#ifndef SkOpts_DEFINED
#define SkOpts_DEFINED

#include "SkRasterPipeline.h"

namespace SkOpts {
    // Declare function pointers here...

#define M(st) +1
    // We can't necessarily express the type of SkJumper stage functions here,
    // so we just use this void(*)(void) as a stand-in.
    using StageFn = void(*)(void);
    extern StageFn stages_highp[SK_RASTER_PIPELINE_STAGES(M)], just_return_highp;
    extern StageFn stages_lowp [SK_RASTER_PIPELINE_STAGES(M)], just_return_lowp;

    extern void (*start_pipeline_highp)(size_t,size_t,size_t,size_t, void**);
    extern void (*start_pipeline_lowp )(size_t,size_t,size_t,size_t, void**);
#undef M
}

#endif //SkOpts_DEFINED
