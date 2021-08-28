// Copyright 2011 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod path_aa;
pub mod path;
pub mod hairline_aa;
pub mod hairline;


use crate::{IntRect, Rect};

use crate::blitter::Blitter;
use crate::geom::ScreenIntRect;


pub fn fill_rect(
    rect: &Rect,
    clip: &ScreenIntRect,
    blitter: &mut dyn Blitter,
) -> Option<()> {
    fill_int_rect(&rect.round(), clip, blitter)
}

fn fill_int_rect(
    rect: &IntRect,
    clip: &ScreenIntRect,
    blitter: &mut dyn Blitter,
) -> Option<()> {
    let rect = rect.intersect(&clip.to_int_rect())?.to_screen_int_rect()?;
    blitter.blit_rect(&rect);
    Some(())
}

pub fn fill_rect_aa(
    rect: &Rect,
    clip: &ScreenIntRect,
    blitter: &mut dyn Blitter,
) -> Option<()> {
    hairline_aa::fill_rect(rect, clip, blitter)
}

