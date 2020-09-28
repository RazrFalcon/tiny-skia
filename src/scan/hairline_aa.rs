// Copyright 2011 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::convert::TryFrom;

use crate::{Rect, ScreenIntRect, AlphaU8, LengthU32};

use crate::blitter::Blitter;
use crate::fdot8::{self, FDot8};
use crate::fixed::{self, Fixed};

#[derive(Copy, Clone, Debug)]
struct FixedRect {
    left: Fixed,
    top: Fixed,
    right: Fixed,
    bottom: Fixed,
}

impl FixedRect {
    fn from_rect(src: &Rect) -> Self {
        FixedRect {
            left: fixed::from_f32(src.left()),
            top: fixed::from_f32(src.top()),
            right: fixed::from_f32(src.right()),
            bottom: fixed::from_f32(src.bottom()),
        }
    }
}


/// Multiplies value by 0..256, and shift the result down 8
/// (i.e. return (value * alpha256) >> 8)
#[inline]
fn alpha_mul(value: AlphaU8, alpha256: i32) -> u8 {
    let a = (i32::from(value) * alpha256) >> 8;
    debug_assert!(a >= 0 && a <= 255);
    a as u8
}


pub fn fill_rect(
    rect: &Rect,
    clip: &ScreenIntRect,
    blitter: &mut dyn Blitter,
) -> Option<()> {
    let rect = rect.intersect(&clip.to_rect())?;
    let fr = FixedRect::from_rect(&rect);
    fill_fixed_rect(&fr, blitter);
    Some(())
}

fn fill_fixed_rect(rect: &FixedRect, blitter: &mut dyn Blitter) {
    fill_dot8(
        fdot8::from_fixed(rect.left),
        fdot8::from_fixed(rect.top),
        fdot8::from_fixed(rect.right),
        fdot8::from_fixed(rect.bottom),
        true,
        blitter,
    )
}

fn fill_dot8(l: FDot8, t: FDot8, r: FDot8, b: FDot8, fill_inner: bool, blitter: &mut dyn Blitter) {
    fn to_alpha(a: i32) -> u8 {
        debug_assert!(a >= 0 && a <= 255);
        a as u8
    }

    // check for empty now that we're in our reduced precision space
    if l >= r || t >= b {
        return;
    }

    let mut top = t >> 8;
    if top == ((b - 1) >> 8) {
        // just one scanline high
        do_scanline(l, top, r, to_alpha(b - t - 1), blitter);
        return;
    }

    if t & 0xFF != 0 {
        do_scanline(l, top, r, to_alpha(256 - (t & 0xFF)), blitter);
        top += 1;
    }

    let bottom = b >> 8;
    let height = bottom - top;
    if let Some(height) = u32::try_from(height).ok().and_then(LengthU32::new) {
        let mut left = l >> 8;
        if left == ((r - 1) >> 8) {
            // just 1-pixel wide
            let left = u32::try_from(left).unwrap();
            let top = u32::try_from(top).unwrap();
            blitter.blit_v(left, top, height, to_alpha(r - l - 1));
        } else {
            if l & 0xFF != 0 {
                {
                    let left = u32::try_from(left).unwrap();
                    let top = u32::try_from(top).unwrap();
                    blitter.blit_v(left, top, height, to_alpha(256 - (l & 0xFF)));
                }

                left += 1;
            }

            let right = r >> 8;
            let width = right - left;
            if fill_inner {
                if let Some(width) = u32::try_from(width).ok().and_then(LengthU32::new) {
                    let left = u32::try_from(left).unwrap();
                    let top = u32::try_from(top).unwrap();
                    let rect = ScreenIntRect::from_xywh_safe(left, top, width, height);
                    blitter.blit_rect(&rect);
                }
            }

            if r & 0xFF != 0 {
                let right = u32::try_from(right).unwrap();
                let top = u32::try_from(top).unwrap();
                blitter.blit_v(right, top, height, to_alpha(r & 0xFF));
            }
        }
    }

    if b & 0xFF != 0 {
        do_scanline(l, bottom, r, to_alpha(b & 0xFF), blitter);
    }
}

fn do_scanline(l: FDot8, top: i32, r: FDot8, alpha: AlphaU8, blitter: &mut dyn Blitter) {
    debug_assert!(l < r);

    let one_len = LengthU32::new(1).unwrap();
    let top = u32::try_from(top).unwrap();

    if (l >> 8) == ((r - 1) >> 8) {
        // 1x1 pixel
        let left = u32::try_from(l >> 8).unwrap();
        blitter.blit_v(left, top, one_len, alpha_mul(alpha, r - l));
        return;
    }

    let mut left = l >> 8;

    if l & 0xFF != 0 {
        {
            let left = u32::try_from(l >> 8).unwrap();
            blitter.blit_v(left, top, one_len, alpha_mul(alpha, 256 - (l & 0xFF)));
        }

        left += 1;
    }

    let right = r >> 8;
    let width = right - left;
    if let Some(width) = u32::try_from(width).ok().and_then(LengthU32::new) {
        let left = u32::try_from(left).unwrap();
        call_hline_blitter(left, top, width, alpha, blitter);
    }

    if r & 0xFF != 0 {
        let right = u32::try_from(right).unwrap();
        blitter.blit_v(right, top, one_len, alpha_mul(alpha, r & 0xFF));
    }
}

fn call_hline_blitter(mut x: u32, y: u32, count: LengthU32, alpha: AlphaU8, blitter: &mut dyn Blitter) {
    const HLINE_STACK_BUFFER: usize = 100;

    let mut runs = [0u16; HLINE_STACK_BUFFER + 1];
    let mut aa = [0u8; HLINE_STACK_BUFFER];

    let mut count = count.get();
    loop {
        // In theory, we should be able to just do this once (outside of the loop),
        // since aa[] and runs[] are supposed" to be const when we call the blitter.
        // In reality, some wrapper-blitters (e.g. RgnClipBlitter) cast away that
        // constness, and modify the buffers in-place. Hence the need to be defensive
        // here and reseed the aa value.
        aa[0] = alpha;

        let mut n = count;
        if n > HLINE_STACK_BUFFER as u32 {
            n = HLINE_STACK_BUFFER as u32;
        }

        debug_assert!(n <= std::u16::MAX as u32);
        runs[0] = n as u16;
        runs[n as usize] = 0;
        blitter.blit_anti_h(x, y, &aa, &runs);
        x += n;

        if n > count || count == 0 {
            break;
        }

        count -= n;
    }
}
