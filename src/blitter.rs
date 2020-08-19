// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{LengthU32, ScreenIntRect};

/// Blitter and its subclasses are responsible for actually writing pixels
/// into memory. Besides efficiency, they handle clipping and antialiasing.
/// A Blitter subclass contains all the context needed to generate pixels
/// for the destination and how src/generated pixels map to the destination.
/// The coordinates passed to the blitX calls are in destination pixel space.
pub trait Blitter {
    /// Blit a horizontal run of one or more pixels.
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32);

    /// Blit a solid rectangle one or more pixels wide.
    fn blit_rect(&mut self, rect: ScreenIntRect);
}
