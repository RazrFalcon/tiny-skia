// Copyright 2012 Google Inc.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Based on skia/tests/StrokeTest.cpp

use tiny_skia::*;

#[test]
fn cubic_1() {
    let mut pb = PathBuilder::new();
    pb.move_to( 51.0161362, 1511.52478);
    pb.cubic_to(51.0161362, 1511.52478,
                51.0161362, 1511.52478,
                51.0161362, 1511.52478);
    let path = pb.finish().unwrap();

    let mut props = StrokeProps::default();
    props.width = 0.394537568;

    assert!(path.stroke(props).is_none());
}

#[test]
fn cubic_2() {
    let mut pb = PathBuilder::new();
    // 51.0161362, 1511.52478
    pb.move_to(f32::from_bits(0x424c1086), f32::from_bits(0x44bcf0cb));
    // 51.0160980, 1511.52478
    // 51.0163651, 1511.52478
    // 51.0166969, 1511.52466
    pb.cubic_to(f32::from_bits(0x424c107c), f32::from_bits(0x44bcf0cb),
                f32::from_bits(0x424c10c2), f32::from_bits(0x44bcf0cb),
                f32::from_bits(0x424c1119), f32::from_bits(0x44bcf0ca));
    let path = pb.finish().unwrap();

    let mut props = StrokeProps::default();
    props.width = 0.394537568;

    assert!(path.stroke(props).is_some());
}

// TODO: test_strokerect

// From skbug.com/6491. The large stroke width can cause numerical instabilities.
#[test]
fn big() {
    // Skia uses `kStrokeAndFill_Style` here, but we do not support it.

    let mut pb = PathBuilder::new();
    pb.move_to(f32::from_bits(0x46380000), f32::from_bits(0xc6380000)); // 11776, -11776
    pb.line_to(f32::from_bits(0x46a00000), f32::from_bits(0xc6a00000)); // 20480, -20480
    pb.line_to(f32::from_bits(0x468c0000), f32::from_bits(0xc68c0000)); // 17920, -17920
    pb.line_to(f32::from_bits(0x46100000), f32::from_bits(0xc6100000)); // 9216, -9216
    pb.line_to(f32::from_bits(0x46380000), f32::from_bits(0xc6380000)); // 11776, -11776
    pb.close();
    let path = pb.finish().unwrap();

    let mut props = StrokeProps::default();
    props.width = 1.49679073e+10;

    assert!(path.stroke(props).is_some());
}
