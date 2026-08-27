# Dual SVG embedding (`<asvg>`)

By default, the original vector is embedded via Word's modern `<asvg>`
extension, with the rasterized PNG kept as a fallback for older viewers. Pass
`--no-embed-svg` to embed only the PNG.

![Flow diagram](../diagrams/01-flow.svg)

Text after the diagram to confirm the paragraph flow is unaffected.
