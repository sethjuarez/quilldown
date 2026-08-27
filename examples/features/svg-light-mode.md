# Light-mode SVG remap

This diagram is authored for a **dark** UI — a near-black background (`#0d1117`)
with light text. By default, quilldown flips each color's lightness before
rasterizing so it reads well on a white Word page, while the indigo accent keeps
its hue.

![Dark-themed flow diagram](../diagrams/02-flow-dark.svg)

With `--no-svg-light-mode`, the dark diagram is embedded as-authored.
