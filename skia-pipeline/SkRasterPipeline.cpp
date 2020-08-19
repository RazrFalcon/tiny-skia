/*
 * Copyright 2016 Google Inc.
 *
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#include "SkOpts.h"

StartPipelineFn build_pipeline(StageList* stages, void** ip) {
    // We'll try to build a lowp pipeline, but if that fails fallback to a highp float pipeline.
    void** reset_point = ip;

    // Stages are stored backwards in fStages, so we reverse here, back to front.
    *--ip = (void*)SkOpts::just_return_lowp;
    for (const StageList* st = stages; st; st = st->prev) {
        if (auto fn = SkOpts::stages_lowp[st->stage]) {
            if (st->ctx) {
                *--ip = st->ctx;
            }
            *--ip = (void*)fn;
        } else {
            ip = reset_point;
            break;
        }
    }
    if (ip != reset_point) {
        return SkOpts::start_pipeline_lowp;
    }

    *--ip = (void*)SkOpts::just_return_highp;
    for (const StageList* st = stages; st; st = st->prev) {
        if (st->ctx) {
            *--ip = st->ctx;
        }
        *--ip = (void*)SkOpts::stages_highp[st->stage];
    }
    return SkOpts::start_pipeline_highp;
}

bool skia_pipe_raster_build_pipeline(StageList *stages, void** ip)
{
    auto fn = build_pipeline((StageList*)stages, ip);
    return fn == SkOpts::start_pipeline_highp;
}

void skia_pipe_raster_run_pipeline(void** program, bool is_highp, unsigned int x, unsigned int y, unsigned int w, unsigned int h)
{
    if (is_highp) {
        SkOpts::start_pipeline_highp(x, y, x+w, y+h, program);
    } else {
        SkOpts::start_pipeline_lowp(x, y, x+w, y+h, program);
    }
}
