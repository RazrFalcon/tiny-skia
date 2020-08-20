// Copyright 2006 The Android Open Source Project
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Point, PathBuilder, Bounds};

const SCALAR_MAX: f32 = 3.402823466e+38;


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum PathVerb {
    Move,
    Line,
    Quad,
    Cubic,
    Close, // TODO: remove?
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
    #[inline]
    pub fn len(&self) -> usize {
        self.verbs.len()
    }

    /// Checks if path is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the bounds of the path's points.
    ///
    /// The value is already calculated.
    #[inline]
    pub fn bounds(&self) -> Bounds {
        self.bounds
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
        !(b.left().get() >= -MAX && b.top().get() >= -MAX && b.right().get() <= MAX && b.bottom().get() <= MAX)
    }

    /// Returns an iterator over path's segments.
    #[inline]
    pub fn segments(&self) -> PathSegmentsIter {
        PathSegmentsIter {
            path: self,
            verb_index: 0,
            points_index: 0,
        }
    }

    #[inline]
    pub(crate) fn edge_iter(&self) -> PathEdgeIter {
        PathEdgeIter {
            path: self,
            verb_index: 0,
            points_index: 0,
            move_to: Point::zero(),
            needs_close_line: false,
            next_is_new_contour: false,
        }
    }

    /// Clears the path and returns a `PathBuilder` that will reuse an allocated memory.
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


/// A path segment.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathSegment {
    Move(Point),
    Line(Point),
    Quad(Point, Point),
    Cubic(Point, Point, Point),
    Close,
}

/// A path segments iterator.
#[allow(missing_debug_implementations)]
pub struct PathSegmentsIter<'a> {
    path: &'a Path,
    verb_index: usize,
    points_index: usize,
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
                    Some(PathSegment::Move(self.path.points[self.points_index - 1]))
                }
                PathVerb::Line => {
                    self.points_index += 1;
                    Some(PathSegment::Line(self.path.points[self.points_index - 1]))
                }
                PathVerb::Quad => {
                    self.points_index += 2;
                    Some(PathSegment::Quad(
                        self.path.points[self.points_index - 2],
                        self.path.points[self.points_index - 1],
                    ))
                }
                PathVerb::Cubic => {
                    self.points_index += 3;
                    Some(PathSegment::Cubic(
                        self.path.points[self.points_index - 3],
                        self.path.points[self.points_index - 2],
                        self.path.points[self.points_index - 1],
                    ))
                }
                PathVerb::Close => {
                    Some(PathSegment::Close)
                }
            }
        } else {
            None
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathEdge {
    Line(Point, Point),
    Quad(Point, Point, Point),
    Cubic(Point, Point, Point, Point),
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
    next_is_new_contour: bool,
}

impl<'a, 'b> PathEdgeIter<'a> {
    fn close_line(&mut self) -> Option<(PathEdge, bool)> {
        self.needs_close_line = false;
        self.next_is_new_contour = true;

        let edge = PathEdge::Line(self.path.points[self.points_index - 1], self.move_to);
        Some((edge, false))
    }
}

impl<'a> Iterator for PathEdgeIter<'a> {
    type Item = (PathEdge, bool);

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

                    let is_new_contour = self.next_is_new_contour;
                    self.next_is_new_contour = false;

                    let edge;
                    match verb {
                        PathVerb::Line => {
                            edge = PathEdge::Line(
                                self.path.points[self.points_index - 1],
                                self.path.points[self.points_index + 0],
                            );
                            self.points_index += 1;
                        }
                        PathVerb::Quad => {
                            edge = PathEdge::Quad(
                                self.path.points[self.points_index - 1],
                                self.path.points[self.points_index + 0],
                                self.path.points[self.points_index + 1],
                            );
                            self.points_index += 2;
                        }
                        PathVerb::Cubic => {
                            edge = PathEdge::Cubic(
                                self.path.points[self.points_index - 1],
                                self.path.points[self.points_index + 0],
                                self.path.points[self.points_index + 1],
                                self.path.points[self.points_index + 2],
                            );
                            self.points_index += 3;
                        }
                        _ => unreachable!(),
                    };

                    Some((edge, is_new_contour))
                }
            }
        } else if self.needs_close_line {
            self.close_line()
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let mut s = String::new();
        for segment in self.segments() {
            match segment {
                PathSegment::Move(p) =>
                    s.write_fmt(format_args!("M {} {} ", p.x, p.y))?,
                PathSegment::Line(p) =>
                    s.write_fmt(format_args!("L {} {} ", p.x, p.y))?,
                PathSegment::Quad(p0, p1) =>
                    s.write_fmt(format_args!("Q {} {} {} {} ", p0.x, p0.y, p1.x, p1.y))?,
                PathSegment::Cubic(p0, p1, p2) =>
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
