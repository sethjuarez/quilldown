# Changelog

## [1.3.0](https://github.com/sethjuarez/quilldown/compare/quilldown-v1.2.0...quilldown-v1.3.0) (2026-09-11)


### Features

* **spec:** harden polyglot parity gates ([7f96d6d](https://github.com/sethjuarez/quilldown/commit/7f96d6dcb4abda48f583b169eab9b3e20c106249))

## [1.2.0](https://github.com/sethjuarez/quilldown/compare/quilldown-v1.1.0...quilldown-v1.2.0) (2026-09-10)


### Features

* **python:** drive L2 document parity with the Rust reference engine ([6647b7a](https://github.com/sethjuarez/quilldown/commit/6647b7a2483b580fd5d2edf970f869d565fcc28d))
* **rust:** preserve math and images in Core IR lowering ([06b5505](https://github.com/sethjuarez/quilldown/commit/06b550596a4d9e7206e4bed1575e39b67b38abc9))

## [1.1.0](https://github.com/sethjuarez/quilldown/compare/quilldown-v1.0.0...quilldown-v1.1.0) (2026-09-09)


### Features

* author spec/*.tsp and green Python runtime (ADR-0001 steps 4-5) ([4863232](https://github.com/sethjuarez/quilldown/commit/4863232e4939b04029ad45cff89aa6db55515102))


### Bug Fixes

* **spec:** project lower_oracle output into spec wire shape ([a36c4cc](https://github.com/sethjuarez/quilldown/commit/a36c4ccdc151adf7c98922505145927211a14e0e))

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
