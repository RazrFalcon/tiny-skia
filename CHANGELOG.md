# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- `PixmapRef` and `PixmapMut`, that can be created from `Pixmap` or from raw data.

### Changed
- `Canvas` no longer owns a `Pixmap`.
- `Canvas::draw_pixmap` and `Pattern::new` accept `PixmapRef` instead of `&Pixmap` now.
- Improve clipping performance.

### Removed
- `Canvas::new`

## 0.2.0 - 2020-11-16
### Changed
- Port to Rust.

## 0.1.0 - 2020-07-04
### Added
- Bindings to a stripped down Skia fork.

[Unreleased]: https://github.com/RazrFalcon/tiny-skia/compare/v0.2.0...HEAD
