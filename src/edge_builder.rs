// Copyright 2011 Google Inc.
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use crate::{Point, Path};

use crate::edge::{Edge, LineEdge, QuadraticEdge, CubicEdge};
use crate::edge_clipper::EdgeClipperIter;
use crate::geom::ScreenIntRect;
use crate::path::PathEdge;
use crate::path_geometry;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Combine {
    No,
    Partial,
    Total,
}


#[derive(Copy, Clone, Debug)]
pub struct ShiftedIntRect {
    shifted: ScreenIntRect,
    shift: i32,
}

impl ShiftedIntRect {
    pub fn new(rect: &ScreenIntRect, shift: i32) -> Option<Self> {
        Some(ShiftedIntRect {
            shifted: ScreenIntRect::from_xywh(
                rect.x() << shift,
                rect.y() << shift,
                rect.width() << shift,
                rect.height() << shift,
            )?,
            shift,
        })
    }

    pub fn shifted(&self) -> &ScreenIntRect {
        &self.shifted
    }

    pub fn recover(&self) -> ScreenIntRect {
        ScreenIntRect::from_xywh(
            self.shifted.x() >> self.shift,
            self.shifted.y() >> self.shift,
            self.shifted.width() >> self.shift,
            self.shifted.height() >> self.shift,
        ).unwrap() // cannot fail, because the original rect was valid
    }
}


pub struct BasicEdgeBuilder {
    edges: Vec<Edge>,
    clip_shift: i32,
}

impl BasicEdgeBuilder {
    pub fn new(clip_shift: i32) -> Self {
        BasicEdgeBuilder {
            edges: Vec::with_capacity(64), // TODO: stack array + fallback
            clip_shift,
        }
    }

    // Skia returns a linked list here, but it's a nightmare to use in Rust,
    // so we're mimicking it with Vec.
    pub fn build_edges(
        path: &Path,
        clip: Option<&ShiftedIntRect>,
        clip_shift: i32,
    ) -> Option<Vec<Edge>> {
        // If we're convex, then we need both edges, even if the right edge is past the clip.
        // let can_cull_to_the_right = !path.isConvex();
        let can_cull_to_the_right = false; // TODO: this

        let mut builder = BasicEdgeBuilder::new(clip_shift);
        builder.build(path, clip, can_cull_to_the_right)?;

        if builder.edges.len() < 2 {
            return None;
        }

        Some(builder.edges)
    }

    // TODO: build_poly
    pub fn build(
        &mut self,
        path: &Path,
        clip: Option<&ShiftedIntRect>,
        can_cull_to_the_right: bool,
    ) -> Option<()> {
        if let Some(ref clip) = clip {
            let clip = clip.recover().to_rect();
            for edges in EdgeClipperIter::new(path, clip, can_cull_to_the_right) {
                for edge in edges {
                    match edge {
                        PathEdge::LineTo(p0, p1) => {
                            if !p0.is_finite() || !p1.is_finite() {
                                return None;
                            }

                            self.push_line(&[p0, p1])
                        }
                        PathEdge::QuadTo(p0, p1, p2) => {
                            if !p0.is_finite() || !p1.is_finite() || !p2.is_finite() {
                                return None;
                            }

                            self.push_quad(&[p0, p1, p2])
                        }
                        PathEdge::CubicTo(p0, p1, p2, p3) => {
                            if !p0.is_finite() || !p1.is_finite() ||
                               !p2.is_finite() || !p3.is_finite()
                            {
                                return None;
                            }

                            self.push_cubic(&[p0, p1, p2, p3])
                        }
                    }
                }
            }
        } else {
            for edge in path.edge_iter() {
                match edge {
                    PathEdge::LineTo(p0, p1) => {
                        self.push_line(&[p0, p1]);
                    }
                    PathEdge::QuadTo(p0, p1, p2) => {
                        let points = [p0, p1, p2];
                        let mut mono_x = [Point::zero(); 5];
                        let n = path_geometry::chop_quad_at_y_extrema(&points, &mut mono_x);
                        for i in 0..=n {
                            self.push_quad(&mono_x[i * 2..]);
                        }
                    }
                    PathEdge::CubicTo(p0, p1, p2, p3) => {
                        let points = [p0, p1, p2, p3];
                        let mut mono_y = [Point::zero(); 10];
                        let n = path_geometry::chop_cubic_at_y_extrema(&points, &mut mono_y);
                        for i in 0..=n {
                            self.push_cubic(&mono_y[i * 3..]);
                        }
                    }
                }
            }
        }

        Some(())
    }

    fn push_line(&mut self, points: &[Point; 2]) {
        if let Some(edge) = LineEdge::new(points[0], points[1], self.clip_shift) {
            let combine = if edge.is_vertical() && !self.edges.is_empty() {
                if let Some(Edge::Line(last)) = self.edges.last_mut() {
                    combine_vertical(&edge, last)
                } else {
                    Combine::No
                }
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

    fn push_quad(&mut self, points: &[Point]) {
        if let Some(edge) = QuadraticEdge::new(points, self.clip_shift) {
            self.edges.push(Edge::Quadratic(edge));
        }
    }

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
