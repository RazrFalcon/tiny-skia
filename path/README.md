# tiny-skia-path
![Build Status](https://github.com/RazrFalcon/tiny-skia/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/tiny-skia-path.svg)](https://crates.io/crates/tiny-skia-path)
[![Documentation](https://docs.rs/tiny-skia-path/badge.svg)](https://docs.rs/tiny-skia-path)
[![Rust 1.51+](https://img.shields.io/badge/rust-1.51+-orange.svg)](https://www.rust-lang.org)

A [tiny-skia](https://github.com/RazrFalcon/tiny-skia) Bezier path implementation.

Provides a memory-efficient Bezier path container, path builder, path stroker and path dasher.

Also provides some basic geometry types, but they will be moved to an external crate eventually.

Note that all types use single precision floats (`f32`), just like [Skia](https://skia.org/).

## License

The same as used by [Skia](https://skia.org/): [New BSD License](./LICENSE)
