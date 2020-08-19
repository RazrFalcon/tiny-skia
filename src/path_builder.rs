// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// NOTE: this is not SkPathBuilder, but rather a reimplementation of SkPath.

use crate::{Point, Bounds, Path};

use crate::checked_geom_ext::BoundsExt;
use crate::path::PathVerb;


/// A path builder.
#[allow(missing_debug_implementations)]
pub struct PathBuilder {
    pub(crate) verbs: Vec<PathVerb>,
    pub(crate) points: Vec<Point>,
    pub(crate) last_move_to_index: usize,
    pub(crate) move_to_required: bool,
}

impl PathBuilder {
    /// Creates a new builder.
    #[inline]
    pub fn new() -> Self {
        PathBuilder {
            verbs: Vec::new(),
            points: Vec::new(),
            last_move_to_index: 0,
            move_to_required: true,
        }
    }

    /// Creates a new builder with a specified capacity.
    ///
    /// Number of points depends on a verb type:
    ///
    /// - Move - 1
    /// - Line - 1
    /// - Quad - 2
    /// - Cubic - 3
    /// - Close - 0
    #[inline]
    pub fn with_capacity(verbs_capacity: usize, points_capacity: usize) -> Self {
        PathBuilder {
            verbs: Vec::with_capacity(verbs_capacity),
            points: Vec::with_capacity(points_capacity),
            last_move_to_index: 0,
            move_to_required: true,
        }
    }

    /// Creates a new `Path` from `Bounds`.
    ///
    /// Never fails since `Bounds` is always valid.
    ///
    /// Segments are created clockwise: TopLeft -> TopRight -> BottomRight -> BottomLeft
    ///
    /// The contour is closed.
    #[inline]
    pub fn from_bound(bounds: Bounds) -> Path {
        let verbs = vec![
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
        ];

        let points = vec![
            Point::from_xy(bounds.left(), bounds.top()),
            Point::from_xy(bounds.right(), bounds.top()),
            Point::from_xy(bounds.right(), bounds.bottom()),
            Point::from_xy(bounds.left(), bounds.bottom()),
        ];

        Path {
            bounds,
            verbs,
            points,
        }
    }

    /// Adds beginning of a contour.
    ///
    /// Multiple continuous MoveTo segments are not allowed.
    /// If the previous segment was also MoveTo, it will be overwritten with the current one.
    pub fn move_to(&mut self, x: f32, y: f32) {
        if let Some(PathVerb::Move) = self.verbs.last() {
            let last_idx = self.points.len() - 1;
            self.points[last_idx] = Point::from_xy(x, y);
        } else {
            self.last_move_to_index = self.points.len();
            self.move_to_required = false;

            self.verbs.push(PathVerb::Move);
            self.points.push(Point::from_xy(x, y));
        }
    }

    #[inline(never)]
    fn inject_move_to_if_needed(&mut self) {
        if self.move_to_required {
            match self.points.get(self.last_move_to_index).cloned() {
                Some(p) => self.move_to(p.x, p.y),
                None => self.move_to(0.0, 0.0),
            }
        }
    }

    /// Adds a line from the last point.
    ///
    /// - If Path is empty - adds Move(0, 0) first.
    /// - If Path ends with Close - adds Move(last_x, last_y) first.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.inject_move_to_if_needed();

        self.verbs.push(PathVerb::Line);
        self.points.push(Point::from_xy(x, y));
    }

    /// Adds a quad curve from the last point to `x`, `y`.
    ///
    /// - If Path is empty - adds Move(0, 0) first.
    /// - If Path ends with Close - adds Move(last_x, last_y) first.
    pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.inject_move_to_if_needed();

        self.verbs.push(PathVerb::Quad);
        self.points.push(Point::from_xy(x1, y1));
        self.points.push(Point::from_xy(x, y));
    }

    /// Adds a cubic curve from the last point to `x`, `y`.
    ///
    /// - If Path is empty - adds Move(0, 0) first.
    /// - If Path ends with Close - adds Move(last_x, last_y) first.
    pub fn cubic_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.inject_move_to_if_needed();

        self.verbs.push(PathVerb::Cubic);
        self.points.push(Point::from_xy(x1, y1));
        self.points.push(Point::from_xy(x2, y2));
        self.points.push(Point::from_xy(x, y));
    }

    /// Closes the current contour.
    ///
    /// A closed contour connects the first and the last Point
    /// with a line, forming a continuous loop.
    ///
    /// Does nothing when Path is empty or already closed.
    ///
    /// Open and closed contour will be filled the same way.
    /// Stroking an open contour will add LineCap at contour's start and end.
    /// Stroking an closed contour will add LineJoin at contour's start and end.
    pub fn close(&mut self) {
        // don't add a close if it's the first verb or a repeat
        if !self.verbs.is_empty() {
            if self.verbs.last().cloned() != Some(PathVerb::Close) {
                self.verbs.push(PathVerb::Close);
            }
        }

        self.move_to_required = true;
    }

    /// Reset the builder.
    ///
    /// Memory is not deallocated.
    pub fn clear(&mut self) {
        self.verbs.clear();
        self.points.clear();
        self.last_move_to_index = 0;
        self.move_to_required = true;
    }

    /// Finishes the builder and returns a `Path`.
    ///
    /// Returns `None` when `Path` is empty or has zero bounds.
    pub fn finish(self) -> Option<Path> {
        // Just a move to? Bail.
        if self.verbs.len() == 1 {
            return None;
        }

        let bounds = Bounds::from_points(&self.points)?;

        Some(Path {
            bounds,
            verbs: self.verbs,
            points: self.points,
        })
    }
}
