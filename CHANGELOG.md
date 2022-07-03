# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [0.7.0] - 2022-07-03
### Added
- `tiny-skia-path` dependency that can be used independently from `tiny-skia`.
  It contains the `tiny-skia` Bezier path implementation, including stroking and dashing.
  As well as all the geometry primitives (like `Point` and `Rect`).

### Changed
- When disabling the `std` feature, one have to enable `no-std-float` feature instead of `libm` now.

## [0.6.6] - 2022-06-23
### Fixed
- Panic in `Rect::round` and `Rect::round_out`.
  Thanks to [@Wardenfar](https://github.com/Wardenfar)

## [0.6.5] - 2022-06-10
### Fixed
- Minimum `arrayref` version.

## [0.6.4] - 2022-06-04
### Fixed
- Panic during non-aliased hairline stroking at the bottom edge of an image.

## [0.6.3] - 2022-02-01
### Fixed
- SourceOver blend mode must not be optimized to Source when ClipPath is present.

## [0.6.2] - 2021-12-30
### Fixed
- `ClipMask::intersect_path` alpha multiplying.

## [0.6.1] - 2021-08-28
### Added
- Support rendering on pixmaps larger than 8191x8191 pixels.
  From now, `Pixmap` is limited only by the amount of memory caller has.
- `Transform::map_points`
- `PathBuilder::push_oval`

## [0.6.0] - 2021-08-21
### Added
- WASM simd128 support. Thanks to [@CryZe](https://github.com/CryZe)

### Changed
- `Transform::post_scale` no longer requires `&mut self`.
- Update `png` crate.

## [0.5.1] - 2021-03-07
### Fixed
- Color memset optimizations should be ignored when clip mask is present.
- `ClipMask::intersect_path` logic.

## [0.5.0] - 2021-03-06
### Added
- `ClipMask::intersect_path`
- no_std support. Thanks to [@CryZe](https://github.com/CryZe)

### Changed
- Reduce `Transform` strictness. It's no longer guarantee to have only finite values,
  therefore we don't have to check each operation.

### Removed
- `Canvas`. Call `Pixmap`/`PixmapMut` drawing methods directly.

## [0.4.2] - 2021-01-23
### Fixed
- Panic during path filling with anti-aliasing because of incorrect edges processing.

## [0.4.1] - 2021-01-19
### Fixed
- Endless loop during stroke dashing.

## [0.4.0] - 2021-01-02
### Changed
- Remove almost all `unsafe`. No performance changes.

## [0.3.0] - 2020-12-20
### Added
- `PixmapRef` and `PixmapMut`, that can be created from `Pixmap` or from raw data.
- `Canvas::set_clip_mask`, `Canvas::get_clip_mask`, `Canvas::take_clip_mask`.

### Changed
- `Canvas` no longer owns a `Pixmap`.
- `Canvas::draw_pixmap` and `Pattern::new` accept `PixmapRef` instead of `&Pixmap` now.
- Improve clipping performance.
- The internal `ClipMask` type become public.

### Fixed
- Panic when path is drawn slightly past the `Pixmap` bounds.

### Removed
- `Canvas::new`

## 0.2.0 - 2020-11-16
### Changed
- Port to Rust.

## 0.1.0 - 2020-07-04
### Added
- Bindings to a stripped down Skia fork.

[Unreleased]: https://github.com/RazrFalcon/tiny-skia/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.6.6...v0.7.0
[0.6.6]: https://github.com/RazrFalcon/tiny-skia/compare/v0.6.5...v0.6.6
[0.6.5]: https://github.com/RazrFalcon/tiny-skia/compare/v0.6.4...v0.6.5
[0.6.4]: https://github.com/RazrFalcon/tiny-skia/compare/v0.6.3...v0.6.4
[0.6.3]: https://github.com/RazrFalcon/tiny-skia/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/RazrFalcon/tiny-skia/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/RazrFalcon/tiny-skia/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/RazrFalcon/tiny-skia/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/RazrFalcon/tiny-skia/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/RazrFalcon/tiny-skia/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.2.0...v0.3.0
