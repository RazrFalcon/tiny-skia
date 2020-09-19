// Copyright 2020 Evgeniy Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{Pixmap, Transform, Path, Paint, StrokeProps, Painter, Point, PathStroker};

/// Provides a high-level rendering API.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Canvas {
    /// A pixmap owned by the canvas.
    pub pixmap: Pixmap,
    /// Canvas's transform.
    pub transform: Transform,

    /// A path stroker used to cache temporary stroking data.
    stroker: PathStroker,
}

impl From<Pixmap> for Canvas {
    #[inline]
    fn from(pixmap: Pixmap) -> Self {
        Canvas {
            pixmap,
            transform: Transform::identity(),
            stroker: PathStroker::new(),
        }
    }
}

impl Canvas {
    /// Fills a path.
    pub fn fill_path(&mut self, path: &Path, paint: &Paint) {
        self.fill_path_impl(path, paint);
    }

    fn fill_path_impl(&mut self, path: &Path, paint: &Paint) -> Option<()> {
        if !self.transform.is_identity() {
            let path = path.clone().transform(&self.transform)?;

            let mut paint = paint.clone();
            paint.shader.transform(&self.transform);

            self.pixmap.fill_path(&path, &paint)
        } else {
            self.pixmap.fill_path(path, paint)
        }
    }

    /// Strokes a path.
    pub fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: StrokeProps) {
        self.stroke_path_impl(path, paint, stroke);
    }

    fn stroke_path_impl(&mut self, path: &Path, paint: &Paint, mut stroke: StrokeProps) -> Option<()> {
        let mut transformed_paint;
        let transformed_path;
        let (path, paint) = if !self.transform.is_identity() {
            stroke.width *= compute_res_scale_for_stroking(&self.transform);

            transformed_paint = paint.clone();
            transformed_paint.shader.transform(&self.transform);

            transformed_path = path.clone().transform(&self.transform)?;
            (&transformed_path, &transformed_paint)
        } else {
            (path, paint)
        };

        let stroked_path = self.stroker.stroke(&path, stroke)?;

        self.pixmap.fill_path(&stroked_path, paint)
    }
}

fn compute_res_scale_for_stroking(ts: &Transform) -> f32 {
    // Not sure how to handle perspective differently, so we just don't try (yet).
    let (sx, ky, kx, sy, _,  _) = ts.get_row();
    let sx = Point::from_xy(sx, kx).length();
    let sy = Point::from_xy(ky, sy).length();
    if sx.is_finite() && sy.is_finite() {
        let scale = sx.max(sy);
        if scale > 0.0 {
            return scale;
        }
    }

    1.0
}
