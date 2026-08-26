# Code highlighting

Fenced code blocks are syntax-highlighted when the fence names a language. Each block gets a
small uppercase language label, and tokens are colored with a light theme that reads well on
the pale code background.

```rust
fn main() {
    // A greeting, in Rust.
    let name = "world";
    println!("Hello, {name}!");
}
```

```python
def greet(name: str) -> str:
    """Return a greeting."""
    return f"Hello, {name}!"
```

A fence with no language falls back to plain, uncolored monospace:

```
plain text, no language, no highlighting
just a monospace block
```
