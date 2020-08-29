// Copyright 2012 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::Point;

/// Converts `&[Point64; N]` into `&[f64; N*2]`.
macro_rules! points64_to_f64s {
    ($pts:expr, $n:expr) => {
        unsafe { &*($pts as *const [Point64; $n] as *const [f64; $n * 2]) }
    };
}

/// Converts `&mut [Point64; N]` into `&mut [f64; N*2]`.
macro_rules! points64_to_f64s_mut {
    ($pts:expr, $n:expr) => {
        unsafe { &mut *(&mut $pts as *mut [Point64; $n] as *mut [f64; $n * 2]) }
    };
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SearchAxis {
    X,
    Y,
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Point64 {
    pub x: f64,
    pub y: f64,
}

impl Point64 {
    #[inline]
    pub fn from_xy(x: f64, y: f64) -> Self {
        Point64 { x, y }
    }

    #[inline]
    pub fn from_point(p: Point) -> Self {
        Point64 {
            x: f64::from(p.x),
            y: f64::from(p.y),
        }
    }

    #[inline]
    pub fn zero() -> Self {
        Point64 { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub fn to_point(&self) -> Point {
        Point::from_xy(self.x as f32, self.y as f32)
    }

    #[inline]
    pub fn axis_coord(&self, axis: SearchAxis) -> f64 {
        match axis {
            SearchAxis::X => self.x,
            SearchAxis::Y => self.y,
        }
    }
}
