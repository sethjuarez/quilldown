# Changelog

## [1.0.0](https://github.com/sethjuarez/quilldown/compare/v0.2.0...v1.0.0) (2026-08-27)


### ⚠ BREAKING CHANGES

* **cli:** the `--svg-light-mode` flag is removed. The light remap is now on by default; pass `--no-svg-light-mode` to keep an SVG's authored colors.
* **cli:** the `--embed-svg` flag is removed. Vector embedding is now on by default; pass `--no-embed-svg` to embed only the rasterized PNG.

### Features

* **cli:** embed SVG vector layer by default ([c92e863](https://github.com/sethjuarez/quilldown/commit/c92e8639d114662b8acebd19fc2df995d78f24f7))
* **cli:** remap SVGs to light mode by default ([7876407](https://github.com/sethjuarez/quilldown/commit/7876407a73dc3e10f16a6bcdd62174aad3c6ac2e))

## [0.2.0](https://github.com/sethjuarez/quilldown/compare/v0.1.0...v0.2.0) (2026-08-27)


### Features

* **math:** render LaTeX as native Word equations (OMML) ([eea085c](https://github.com/sethjuarez/quilldown/commit/eea085c1408a481be402a12704afa5d21d004b6d))
* **math:** render LaTeX math as embedded typeset equations ([03dccac](https://github.com/sethjuarez/quilldown/commit/03dccacc444ade4fc193e52f8f2d3a87ca715291))


### Bug Fixes

* **math:** grow line height so inline equations aren't clipped in Word ([2e453c0](https://github.com/sethjuarez/quilldown/commit/2e453c08340c16d0f5a67972b62010360d84419a))
* **math:** stop clipping tall equation glyphs ([b04b41b](https://github.com/sethjuarez/quilldown/commit/b04b41b622b6621e117741ee1dd9e6f5f7db221d))
