# Changelog

## [0.2.0](https://github.com/sethjuarez/quilldown/compare/quilldown-python-v0.1.1...quilldown-python-v0.2.0) (2026-09-10)


### Features

* **python:** drive L2 document parity with the Rust reference engine ([6647b7a](https://github.com/sethjuarez/quilldown/commit/6647b7a2483b580fd5d2edf970f869d565fcc28d))
* **python:** preserve math and images in lowering ([ead6ac8](https://github.com/sethjuarez/quilldown/commit/ead6ac8ccc8751677fc210a6f854cab6e62adcfd))
* **python:** render real Word constructs to reach L2 document parity ([6a56e4f](https://github.com/sethjuarez/quilldown/commit/6a56e4f658eb8e54795abb255d37100fd9a153d0))

## [0.1.1](https://github.com/sethjuarez/quilldown/compare/quilldown-python-v0.1.0...quilldown-python-v0.1.1) (2026-09-09)


### Documentation

* **python:** tighten the runtime README ([a6a24d8](https://github.com/sethjuarez/quilldown/commit/a6a24d80ceb5570291287e337d43dbb9076dfc06))
* **python:** tighten the runtime README ([e575ec8](https://github.com/sethjuarez/quilldown/commit/e575ec839764125657b580527a2a13e5fcdb1b5d))

## 0.1.0 (2026-09-09)


### Features

* author spec/*.tsp and green Python runtime (ADR-0001 steps 4-5) ([4863232](https://github.com/sethjuarez/quilldown/commit/4863232e4939b04029ad45cff89aa6db55515102))
* **python:** add uv-managed Python runtime passing Core conformance ([90fde38](https://github.com/sethjuarez/quilldown/commit/90fde388d28486b29787d4df204dfa6de93f2cbc))
* **python:** autolink bare URLs, www, and emails to match comrak ([39fd350](https://github.com/sethjuarez/quilldown/commit/39fd350768d2576ed438cce5c56f7ac821d2abaa))
* **python:** drop leading YAML front matter to match comrak ([2787194](https://github.com/sethjuarez/quilldown/commit/27871949082afd2f557cfc2ce361980db2cbcd75))
* **python:** legalize comrak dollar-backtick code math to text ([1506568](https://github.com/sethjuarez/quilldown/commit/1506568542fe5fa6592fd09c3262967842f7f825))
* **python:** legalize footnotes to match the Rust oracle ([cbe42d5](https://github.com/sethjuarez/quilldown/commit/cbe42d5c58337bfc8c7fda299e22158354dbb56f))
* **python:** legalize GFM alerts to unwrapped body blocks ([bcda104](https://github.com/sethjuarez/quilldown/commit/bcda1044c5dd62bee05f9f35d5ecfc02cedfd322))
* **python:** legalize GFM subscript with a comrak-faithful ~ pairing pass ([9d2b1b7](https://github.com/sethjuarez/quilldown/commit/9d2b1b7c3ae03a03d2cd0ae263c858eac22d8a4b))
* **python:** legalize GFM superscript by flattening to inner content ([27f7e79](https://github.com/sethjuarez/quilldown/commit/27f7e793cbc1849a95d3c0ed948cb247542b89f1))
* **python:** legalize images to alt text to match the Rust oracle ([8c2f120](https://github.com/sethjuarez/quilldown/commit/8c2f120a3feb87eb233563c35798bf7518efafcf))
* **python:** legalize inline and display math to match the Rust oracle ([66f3bdb](https://github.com/sethjuarez/quilldown/commit/66f3bdb5a1679e85de8efb0102ff3162c7e0a02b))


### Bug Fixes

* **python:** legalize comrak cross-marker delimiter removal for tilde spans ([f0a5fd8](https://github.com/sethjuarez/quilldown/commit/f0a5fd80f8f2cc859b6002f2f30d1263a02f0a27))
* **python:** unified interleaved delimiter pass for cross-marker emphasis parity ([9a4a83b](https://github.com/sethjuarez/quilldown/commit/9a4a83b8f726e3b9568a25f9739c82b3178ff7e6))


### Documentation

* **python:** document mixed-marker re-pairing exclusion found in duck review ([ad3248f](https://github.com/sethjuarez/quilldown/commit/ad3248f50027a2bdde10fd8afec0ceaeef6d8064))
