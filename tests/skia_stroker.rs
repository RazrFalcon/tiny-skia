// Copyright 2014 Google Inc.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on skia/tests/StrokerTest.cpp

use tiny_skia::*;

// TODO: other test

#[test]
fn quad_stroker_one_off() {
    let mut pb = PathBuilder::new();
    pb.move_to(f32::from_bits(0x43c99223), f32::from_bits(0x42b7417e));
    pb.quad_to(f32::from_bits(0x4285d839), f32::from_bits(0x43ed6645),
               f32::from_bits(0x43c941c8), f32::from_bits(0x42b3ace3));
    let path = pb.finish().unwrap();
    assert!(path.stroke(StrokeProps::default().set_width(164.683548)).is_some());
}

#[test]
fn cubic_stroker_one_off() {
    let mut pb = PathBuilder::new();
    pb.move_to(f32::from_bits(0x433f5370), f32::from_bits(0x43d1f4b3));
    pb.cubic_to(f32::from_bits(0x4331cb76), f32::from_bits(0x43ea3340),
                f32::from_bits(0x4388f498), f32::from_bits(0x42f7f08d),
                f32::from_bits(0x43f1cd32), f32::from_bits(0x42802ec1));
    let path = pb.finish().unwrap();
    assert!(path.stroke(StrokeProps::default().set_width(42.835968)).is_some());
}
