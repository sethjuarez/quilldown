# Changelog

## [1.4.0](https://github.com/sethjuarez/quilldown/compare/quilldown-v1.3.1...quilldown-v1.4.0) (2026-10-05)


### Features

* **ir:** preserve gfm alert callouts ([183c787](https://github.com/sethjuarez/quilldown/commit/183c787ffc6bb2be631f7945ea14c0c21a3a88f0))
* **render:** add captions parity ([fa44f66](https://github.com/sethjuarez/quilldown/commit/fa44f66a56fc3ce2bd462480903ab3c74a514334))
* **render:** add native math parity ([d377e62](https://github.com/sethjuarez/quilldown/commit/d377e62bc22066afe25e153742d78890ab214149))
* **render:** add svg parity ([5582464](https://github.com/sethjuarez/quilldown/commit/558246420cfb581b2544751039e0261ecee44565))
* **render:** add toc parity ([32994af](https://github.com/sethjuarez/quilldown/commit/32994afdd5bc9288bd1720cc00da27db80d14fa5))
* **render:** complete option surface parity ([35e791c](https://github.com/sethjuarez/quilldown/commit/35e791c90e70446acd6fbf8d79b05103012e2ccb))
* **render:** complete Python DOCX parity ([2a52f02](https://github.com/sethjuarez/quilldown/commit/2a52f025e5f6dab5f608d16508dce667cf3ab6dc))
* **render:** support page and theme options ([9750f44](https://github.com/sethjuarez/quilldown/commit/9750f44d6e9c46851b28eaf6d8e68fe1dfdd3e82))

## [1.3.1](https://github.com/sethjuarez/quilldown/compare/quilldown-v1.3.0...quilldown-v1.3.1) (2026-09-25)


### Bug Fixes

* **python:** emit strict-valid docx packages ([129a9f9](https://github.com/sethjuarez/quilldown/commit/129a9f9aa33ffcd091346b0cb3af437d1f872b07))
* **python:** emit strict-valid docx packages ([03e5e58](https://github.com/sethjuarez/quilldown/commit/03e5e58f6c4ab042ccd6a412fe6b828d0add6ef3))

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
