# tiny-skia-geom
![Build Status](https://github.com/RazrFalcon/tiny-skia/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/tiny-skia-geom.svg)](https://crates.io/crates/tiny-skia-geom)
[![Documentation](https://docs.rs/tiny-skia-geom/badge.svg)](https://docs.rs/tiny-skia-geom)
[![Rust 1.46+](https://img.shields.io/badge/rust-1.46+-orange.svg)](https://www.rust-lang.org)

`tiny-skia-geom` is a collection of geometry primitives used by
[tiny-skia](https://github.com/RazrFalcon/tiny-skia).

Almost all types are immutable an validated on creation.

Unlike other crates that provide a Bezier path container, this one supports stroking and dashing,
which takes most of the code.

Note that all types use single precision floats (`f32`), just like [Skia](https://skia.org/).

## License

The same as used by [Skia](https://skia.org/): [New BSD License](./LICENSE)
