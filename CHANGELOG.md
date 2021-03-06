# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Fixed
- Color memset optimizations should be ignored when clip mask is present.

## [0.5.0] - 2021-03-06
### Added
- `ClipMask::intersect_path`
- no_std support.

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

[Unreleased]: https://github.com/RazrFalcon/tiny-skia/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/RazrFalcon/tiny-skia/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/RazrFalcon/tiny-skia/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RazrFalcon/tiny-skia/compare/v0.2.0...v0.3.0
