# Swappable Themes

Themes restyle a document without touching the Markdown: the heading accent, the
[hyperlink](https://example.com) color, the code-block fill, and the syntax-highlight palette
all come from the selected preset.

## A heading uses the theme accent

Body text stays in the theme body font, while inline `code` uses the theme monospace font.

```rust
fn main() {
    println!("themed code block");
}
```

> A block quote keeps its neutral styling regardless of theme.
