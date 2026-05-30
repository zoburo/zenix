# zenix-derive

Proc-macro crate for [zenix](https://github.com/zoburo/zenix). You usually don't
need to depend on this directly — re-exported through `zenix::FromRequest`.

## Derive macros

### `#[derive(FromRequest)]`

Generates a [`zenix::extract::FromRequest`] implementation from per-field
`#[zenix(from = "...")]` annotations, enabling `ctx.bind::<T>()`.

| Annotation                 | Source            |
| -------------------------- | ----------------- |
| `#[zenix(from = "body")]`  | JSON request body |
| `#[zenix(from = "query")]` | URL query string  |
| `#[zenix(from = "form")]`  | URL-encoded body  |

```rust
use zenix::FromRequest;

#[derive(FromRequest)]
struct CreateUser {
    #[zenix(from = "body")]
    name: String,

    #[zenix(from = "query")]
    token: String,
}
```

Each field type must implement `serde::Deserialize`. Fields without an annotation
default to `"body"`.
