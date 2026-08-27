# Math

`$...$` and `$$...$$` math (and fenced ` ```math ` blocks) are converted to
native Word equations (OMML). They reflow with the text, recolor in dark mode,
and stay editable — no rasterized images. LaTeX that can't be represented
degrades to its literal source.

## Inline math

The mass–energy equivalence is $E = mc^2$, and the golden ratio satisfies
$\varphi = \frac{1 + \sqrt{5}}{2}$.

Code-style inline math also works: the sum $`\sum_{i=1}^{n} i = \frac{n(n+1)}{2}`$
appears mid-sentence.

## Display math

A standalone display equation:

$$\int_{a}^{b} f(x)\,dx = F(b) - F(a)$$

And another:

$$e^{i\pi} + 1 = 0$$

## Fenced math block

```math
\begin{aligned}
a^2 + b^2 &= c^2 \\
\cos^2\theta + \sin^2\theta &= 1
\end{aligned}
```
