// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Point, PathBuilder, Bounds, Transform, StrokeProps, PathStroker};

use crate::safe_geom_ext::{BoundsExt, TransformExt};
use crate::scalar::SCALAR_MAX;


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum PathVerb {
    Move,
    Line,
    Quad,
    Cubic,
    Close,
}


/// A Bezier path.
///
/// Can be created via `PathBuilder`.
///
/// # Guarantees
///
/// - Has a valid, precomputed bounds.
/// - All points are finite.
/// - Has at least two segments.
/// - Each contour starts with a Move.
/// - No duplicated Move.
/// - No duplicated Close.
#[derive(Clone, PartialEq)]
pub struct Path {
    pub(crate) verbs: Vec<PathVerb>,
    pub(crate) points: Vec<Point>,
    pub(crate) bounds: Bounds,
}

impl Path {
    /// Returns the number of segments in the path.
    pub fn len(&self) -> usize {
        self.verbs.len()
    }

    /// Checks if path is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the bounds of the path's points.
    ///
    /// The value is already calculated.
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    /// Returns a transformed in-place path.
    ///
    /// Some points may become NaN/inf therefore this method can fail.
    pub fn transform(mut self, ts: &Transform) -> Option<Self> {
        ts.map_points(&mut self.points);

        // Update bounds.
        self.bounds = Bounds::from_points(&self.points)?;

        Some(self)
    }

    /// Produces a stroked path.
    ///
    /// This just a shorthand for:
    ///
    /// ```ignore
    /// let mut stroker = PathStroker::new();
    /// stroker.stroke(self, props);
    /// ```
    ///
    /// Returns `None` when:
    ///
    /// - `width` <= 0 or Nan/inf
    /// - `miter_limit` < 4 or Nan/inf
    /// - produced stroke has invalid bounds
    pub fn stroke(&self, props: StrokeProps) -> Option<Self> {
        let mut stroker = PathStroker::new();
        stroker.stroke(self, props)
    }

    /// Sometimes in the drawing pipeline, we have to perform math on path coordinates, even after
    /// the path is in device-coordinates. Tessellation and clipping are two examples. Usually this
    /// is pretty modest, but it can involve subtracting/adding coordinates, or multiplying by
    /// small constants (e.g. 2,3,4). To try to preflight issues where these optionations could turn
    /// finite path values into infinities (or NaNs), we allow the upper drawing code to reject
    /// the path if its bounds (in device coordinates) is too close to max float.
    pub(crate) fn is_too_big_for_math(&self) -> bool {
        // This value is just a guess. smaller is safer, but we don't want to reject largish paths
        // that we don't have to.
        const SCALE_DOWN_TO_ALLOW_FOR_SMALL_MULTIPLIES: f32 = 0.25;
        const MAX: f32 = SCALAR_MAX * SCALE_DOWN_TO_ALLOW_FOR_SMALL_MULTIPLIES;

        let b = self.bounds;

        // use ! expression so we return true if bounds contains NaN
        !(b.left() >= -MAX && b.top() >= -MAX && b.right() <= MAX && b.bottom() <= MAX)
    }

    /// Returns an iterator over path's segments.
    pub fn segments(&self) -> PathSegmentsIter {
        PathSegmentsIter {
            path: self,
            verb_index: 0,
            points_index: 0,
            is_auto_close: false,
            last_move_to: Point::zero(),
            last_point: Point::zero(),
        }
    }

    pub(crate) fn edge_iter(&self) -> PathEdgeIter {
        PathEdgeIter {
            path: self,
            verb_index: 0,
            points_index: 0,
            move_to: Point::zero(),
            needs_close_line: false,
        }
    }

    /// Clears the path and returns a `PathBuilder` that will reuse an allocated memory.
    #[inline]
    pub fn clear(mut self) -> PathBuilder {
        self.verbs.clear();
        self.points.clear();

        PathBuilder {
            verbs: self.verbs,
            points: self.points,
            last_move_to_index: 0,
            move_to_required: true,
        }
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let mut s = String::new();
        for segment in self.segments() {
            match segment {
                PathSegment::MoveTo(p) =>
                    s.write_fmt(format_args!("M {} {} ", p.x, p.y))?,
                PathSegment::LineTo(p) =>
                    s.write_fmt(format_args!("L {} {} ", p.x, p.y))?,
                PathSegment::QuadTo(p0, p1) =>
                    s.write_fmt(format_args!("Q {} {} {} {} ", p0.x, p0.y, p1.x, p1.y))?,
                PathSegment::CubicTo(p0, p1, p2) =>
                    s.write_fmt(format_args!("C {} {} {} {} {} {} ", p0.x, p0.y, p1.x, p1.y, p2.x, p2.y))?,
                PathSegment::Close =>
                    s.write_fmt(format_args!("Z "))?,
            }
        }

        s.pop(); // ' '

        f.debug_struct("Path")
            .field("segments", &s)
            .field("bounds", &self.bounds)
            .finish()
    }
}


/// A path segment.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
    Close,
}

impl PathSegment {
    /// Creates a new MoveTo segment from coordinates.
    pub fn new_move_to(x: f32, y: f32) -> Self {
        PathSegment::MoveTo(Point::from_xy(x, y))
    }

    /// Creates a new LineTo segment from coordinates.
    pub fn new_line_to(x: f32, y: f32) -> Self {
        PathSegment::LineTo(Point::from_xy(x, y))
    }

    /// Creates a new QuadTo segment from coordinates.
    pub fn new_quad_to(x1: f32, y1: f32, x: f32, y: f32) -> Self {
        PathSegment::QuadTo(Point::from_xy(x1, y1), Point::from_xy(x, y))
    }

    /// Creates a new CubicTo segment from coordinates.
    pub fn new_cubic_to(x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) -> Self {
        PathSegment::CubicTo(Point::from_xy(x1, y1), Point::from_xy(x2, y2), Point::from_xy(x, y))
    }

    /// Creates a new Close segment from coordinates.
    pub fn new_close() -> Self {
        PathSegment::Close
    }
}


/// A path segments iterator.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct PathSegmentsIter<'a> {
    path: &'a Path,
    verb_index: usize,
    points_index: usize,

    is_auto_close: bool,
    last_move_to: Point,
    last_point: Point,
}

impl<'a> PathSegmentsIter<'a> {
    /// Sets the auto closing mode. Off by default.
    ///
    /// When enabled, emits an additional `PathSegment::Line` from the current position
    /// to the previous `PathSegment::Move`. And only then emits `PathSegment::Close`.
    pub fn set_auto_close(&mut self, flag: bool) {
        self.is_auto_close = flag;
    }

    pub(crate) fn auto_close(&mut self) -> PathSegment {
        if self.is_auto_close && self.last_point != self.last_move_to {
            self.verb_index -= 1;
            PathSegment::LineTo(self.last_move_to)
        } else {
            PathSegment::Close
        }
    }

    pub(crate) fn has_valid_tangent(&self) -> bool {
        let mut iter = self.clone();
        while let Some(segment) = iter.next() {
            match segment {
                PathSegment::MoveTo(_) => {
                    return false;
                }
                PathSegment::LineTo(p) => {
                    if iter.last_point == p {
                        continue;
                    }

                    return true;
                }
                PathSegment::QuadTo(p1, p2) => {
                    if iter.last_point == p1 && iter.last_point == p2 {
                        continue;
                    }

                    return true;
                }
                PathSegment::CubicTo(p1, p2, p3) => {
                    if iter.last_point == p1 && iter.last_point == p2 && iter.last_point == p3 {
                        continue;
                    }

                    return true;
                }
                PathSegment::Close => {
                    return false;
                }
            }
        }

        false
    }
}

impl<'a> Iterator for PathSegmentsIter<'a> {
    type Item = PathSegment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.verb_index < self.path.verbs.len() {
            let verb = self.path.verbs[self.verb_index];
            self.verb_index += 1;

            match verb {
                PathVerb::Move => {
                    self.points_index += 1;
                    self.last_move_to = self.path.points[self.points_index - 1];
                    self.last_point = self.last_move_to;
                    Some(PathSegment::MoveTo(self.last_move_to))
                }
                PathVerb::Line => {
                    self.points_index += 1;
                    self.last_point = self.path.points[self.points_index - 1];
                    Some(PathSegment::LineTo(self.last_point))
                }
                PathVerb::Quad => {
                    self.points_index += 2;
                    self.last_point = self.path.points[self.points_index - 1];
                    Some(PathSegment::QuadTo(
                        self.path.points[self.points_index - 2],
                        self.last_point,
                    ))
                }
                PathVerb::Cubic => {
                    self.points_index += 3;
                    self.last_point = self.path.points[self.points_index - 1];
                    Some(PathSegment::CubicTo(
                        self.path.points[self.points_index - 3],
                        self.path.points[self.points_index - 2],
                        self.last_point
                    ))
                }
                PathVerb::Close => {
                    let seg = self.auto_close();
                    self.last_point = self.last_move_to;
                    Some(seg)
                }
            }
        } else {
            None
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathEdge {
    LineTo(Point, Point),
    QuadTo(Point, Point, Point),
    CubicTo(Point, Point, Point, Point),
}

/// Lightweight variant of PathIter that only returns segments (e.g. lines/quads).
///
/// Does not return Move or Close. Always "auto-closes" each contour.
pub struct PathEdgeIter<'a> {
    path: &'a Path,
    verb_index: usize,
    points_index: usize,
    move_to: Point,
    needs_close_line: bool,
}

impl<'a, 'b> PathEdgeIter<'a> {
    fn close_line(&mut self) -> Option<PathEdge> {
        self.needs_close_line = false;

        let edge = PathEdge::LineTo(self.path.points[self.points_index - 1], self.move_to);
        Some(edge)
    }
}

impl<'a> Iterator for PathEdgeIter<'a> {
    type Item = PathEdge;

    fn next(&mut self) -> Option<Self::Item> {
        if self.verb_index < self.path.verbs.len() {
            let verb = self.path.verbs[self.verb_index];
            self.verb_index += 1;

            match verb {
                PathVerb::Move => {
                    if self.needs_close_line {
                        let res = self.close_line();
                        self.move_to = self.path.points[self.points_index];
                        self.points_index += 1;
                        return res;
                    }

                    self.move_to = self.path.points[self.points_index];
                    self.points_index += 1;
                    self.next()
                }
                PathVerb::Close => {
                    if self.needs_close_line {
                        return self.close_line();
                    }

                    self.next()
                }
                _ => {
                    // Actual edge.
                    self.needs_close_line = true;

                    let edge;
                    match verb {
                        PathVerb::Line => {
                            edge = PathEdge::LineTo(
                                self.path.points[self.points_index - 1],
                                self.path.points[self.points_index + 0],
                            );
                            self.points_index += 1;
                        }
                        PathVerb::Quad => {
                            edge = PathEdge::QuadTo(
                                self.path.points[self.points_index - 1],
                                self.path.points[self.points_index + 0],
                                self.path.points[self.points_index + 1],
                            );
                            self.points_index += 2;
                        }
                        PathVerb::Cubic => {
                            edge = PathEdge::CubicTo(
                                self.path.points[self.points_index - 1],
                                self.path.points[self.points_index + 0],
                                self.path.points[self.points_index + 1],
                                self.path.points[self.points_index + 2],
                            );
                            self.points_index += 3;
                        }
                        _ => unreachable!(),
                    };

                    Some(edge)
                }
            }
        } else if self.needs_close_line {
            self.close_line()
        } else {
            None
        }
    }
}
