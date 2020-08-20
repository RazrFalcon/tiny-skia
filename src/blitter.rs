// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{LengthU32, ScreenIntRect};

/// Blitter is responsible for actually writing pixels into memory.
///
/// Besides efficiency, they handle clipping and antialiasing.
/// An object that implements Blitter contains all the context needed to generate pixels
/// for the destination and how src/generated pixels map to the destination.
/// The coordinates passed to the `blit_*` calls are in destination pixel space.
pub trait Blitter {
    /// Blits a horizontal run of one or more pixels.
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32);

    /// Blits a solid rectangle one or more pixels wide.
    fn blit_rect(&mut self, rect: ScreenIntRect);
}
