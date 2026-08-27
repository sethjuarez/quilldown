# Table header rows

A plain data table. Its first row is the header and should repeat when the
table breaks across pages.

| Fruit  | Color  | Count |
| ------ | ------ | ----- |
| Apple  | Red    | 3     |
| Lemon  | Yellow | 5     |
| Grape  | Purple | 24    |

A fenced code block renders inside a 1×1 wrapper table — it must **not** be
treated as a header row.

```rust
fn main() {
    println!("hello");
}
```

An alert callout is also a 1×1 wrapper table. The data table nested inside it
still gets a proper repeating header, but the alert's own wrapper row does not.

> [!NOTE]
> Nested table inside an alert:
>
> | Key | Value |
> | --- | ----- |
> | a   | 1     |
> | b   | 2     |
