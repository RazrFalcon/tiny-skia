// Copyright 2011 Google Inc.
// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Point, Path};

use crate::edge::{Edge, LineEdge, QuadraticEdge, CubicEdge};
use crate::geometry;
use crate::path::PathEdge;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Combine {
    No,
    Partial,
    Total,
}

pub struct BasicEdgeBuilder {
    edges: Vec<Edge>,
    clip_shift: i32,
}

impl BasicEdgeBuilder {
    #[inline]
    pub fn new(clip_shift: i32) -> Self {
        BasicEdgeBuilder {
            edges: Vec::with_capacity(64), // TODO: stack array + fallback
            clip_shift,
        }
    }

    // Skia returns a linked list here, but it's a nightmare to use in Rust,
    // so we're mimicking it with Vec.
    #[inline]
    pub fn build_edges(path: &Path, clip_shift: i32) -> Option<Vec<Edge>> {
        let mut builder = BasicEdgeBuilder::new(clip_shift);
        builder.build(path);

        if builder.edges.is_empty() {
            return None;
        }

        debug_assert!(builder.edges.len() != 1);

        Some(builder.edges)
    }

    pub fn build(&mut self, path: &Path) {
        for (edge, _) in path.edge_iter() {
            match edge {
                PathEdge::Line(p0, p1) => {
                    self.push_line(&[p0, p1]);
                }
                PathEdge::Quad(p0, p1, p2) => {
                    let points = [p0, p1, p2];
                    let mut mono_x = [Point::zero(); 5];
                    let n = geometry::chop_quad_at_y_extrema(&points, &mut mono_x);
                    for i in 0..=n {
                        self.push_quad(&mono_x[i * 2..]);
                    }
                }
                PathEdge::Cubic(p0, p1, p2, p3) => {
                    let points = [p0, p1, p2, p3];
                    let mut mono_y = [Point::zero(); 10];
                    let n = geometry::chop_cubic_at_y_extrema(&points, &mut mono_y);
                    for i in 0..=n {
                        self.push_cubic(&mono_y[i * 3..]);
                    }
                }
            }
        }
    }

    fn push_line(&mut self, points: &[Point; 2]) {
        if let Some(edge) = LineEdge::new(points[0], points[1], self.clip_shift) {
            let combine = if edge.is_vertical() && !self.edges.is_empty() {
                combine_vertical(&edge, self.edges.last_mut().unwrap().as_line_mut())
            } else {
                Combine::No
            };

            match combine {
                Combine::Total => { self.edges.pop(); },
                Combine::Partial => {}
                Combine::No => self.edges.push(Edge::Line(edge)),
            }
        }
    }

    #[inline]
    fn push_quad(&mut self, points: &[Point]) {
        if let Some(edge) = QuadraticEdge::new(points, self.clip_shift) {
            self.edges.push(Edge::Quadratic(edge));
        }
    }

    #[inline]
    fn push_cubic(&mut self, points: &[Point]) {
        if let Some(edge) = CubicEdge::new(points, self.clip_shift) {
            self.edges.push(Edge::Cubic(edge));
        }
    }
}

fn combine_vertical(edge: &LineEdge, last: &mut LineEdge) -> Combine {
    if last.dx != 0 || edge.x != last.x {
        return Combine::No;
    }

    if edge.winding == last.winding {
        return if edge.last_y + 1 == last.first_y {
            last.first_y = edge.first_y;
            Combine::Partial
        } else if edge.first_y == last.last_y + 1 {
            last.last_y = edge.last_y;
            Combine::Partial
        } else {
            Combine::No
        };
    }

    if edge.first_y == last.first_y {
        return if edge.last_y == last.last_y {
            Combine::Total
        } else if edge.last_y < last.last_y {
            last.first_y = edge.last_y + 1;
            Combine::Partial
        } else {
            last.first_y = last.last_y + 1;
            last.last_y = edge.last_y;
            last.winding = edge.winding;
            Combine::Partial
        };
    }

    if edge.last_y == last.last_y {
        if edge.first_y > last.first_y {
            last.last_y = edge.first_y - 1;
        } else {
            last.last_y = last.first_y - 1;
            last.first_y = edge.first_y;
            last.winding = edge.winding;
        }

        return Combine::Partial;
    }

    Combine::No
}
