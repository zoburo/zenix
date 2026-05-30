# zenix

High-performance and ergonomic Rust HTTP framework inspired by [Go Fiber](https://gofiber.io/).

Built on top of [`hyper`](https://github.com/hyperium/hyper) and [`tokio`](https://tokio.rs), zenix provides a simple, expressive API for building async web services with minimal boilerplate.

## Installation

Add zenix to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
zenix = { version = "0.0.1" }
```

> **Note:** zenix requires `#[tokio::main]` since it uses Tokio for its async runtime.

## Quickstart

A minimal "Hello, World!" server — start here to see zenix in action:

```rust
use zenix::{Context, Server};

#[tokio::main]
async fn main() {
    let mut server = Server::new();

    server.get("/", |ctx: Context| ctx.string("Hello, World!"));

    server.listen(":3000").await.unwrap();
}
```

Run with `cargo run` and visit `http://localhost:3000`.

## Features

### Routing — register route handlers by HTTP method

zenix maps incoming requests to handlers using exact path matching. Register handlers with the method name directly:

```rust
server.get("/", handler);
server.post("/users", handler);
server.put("/users/:id", handler);
server.delete("/users/:id", handler);
```

Each handler is a closure or function with the signature `fn(Context) -> Response`, providing full access to the request context and response builder.

### Request binding (`#[derive(FromRequest)]`) — deserialize request data into structs

Use the `FromRequest` derive macro to automatically extract and deserialize request data into strongly-typed Rust structs. Each field can be annotated with its source location:

```rust
use zenix::FromRequest;

#[derive(FromRequest)]
struct CreateUser {
    #[zenix(from = "body")]
    name: String,

    #[zenix(from = "query")]
    token: String,
}

fn handler(ctx: Context) -> Response {
    let input: CreateUser = ctx.bind().unwrap();
    ctx.json(r#"{"ok":true}"#)
}
```

The following source annotations are supported:

| Annotation                 | Source            |
| -------------------------- | ----------------- |
| `#[zenix(from = "body")]`  | JSON request body |
| `#[zenix(from = "query")]` | URL query string  |
| `#[zenix(from = "form")]`  | URL-encoded body  |

Fields without an explicit annotation default to `"body"`.

### Validation — validate incoming data with [`garde`](https://docs.rs/garde)

Combine `#[derive(Validate)]` from garde with `#[derive(FromRequest)]` to seamlessly validate deserialized data. Validation errors are automatically formatted into structured JSON responses:

```rust
use garde::Validate;
use zenix::{Context, FromRequest, Response, Server};

#[derive(Validate, FromRequest)]
struct CreateUser {
    #[zenix(from = "body")]
    #[garde(length(min = 1, max = 100))]
    name: String,

    #[zenix(from = "body")]
    #[garde(email)]
    email: String,
}

fn handler(ctx: Context) -> Response {
    let input: CreateUser = match ctx.bind() {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if let Err(report) = input.validate() {
        return ctx.status(StatusCode::UNPROCESSABLE_ENTITY)
            .json(report.to_json());
    }

    ctx.json(r#"{"created":"ok"}"#)
}
```

The `ReportExt` trait (re-exported from `zenix::validate`) adds a `to_json()` method to `garde::Report`, producing field-grouped error messages like:

```json
{
  "name": ["name is required"],
  "email": ["email is not a valid email address"]
}
```

### Responses — build HTTP responses with convenience methods

The `Context` object provides builder methods for common response types, all defaulting to HTTP 200:

```rust
ctx.string("plain text");                          // 200 text/plain
ctx.json(r#"{"key":"value"}"#);                    // 200 application/json
ctx.html("<h1>title</h1>");                        // 200 text/html
ctx.status(StatusCode::CREATED).json(body);        // custom status
```

For full control, you can construct a `hyper::Response<Full<Bytes>>` directly, since `Response` is a type alias for it.

### Request context — inspect the incoming request

Every handler receives a `Context` that exposes the raw request parts:

```rust
fn handler(ctx: Context) -> Response {
    let method = ctx.method();       // HTTP method
    let path   = ctx.path();         // URI path
    let body   = ctx.body();         // Pre-read body bytes
    let qs     = ctx.query_string(); // Raw query string
    // ...
}
```

## Examples

The repository includes ready-to-run examples under `examples/`:

```shell
# Basic hello-world server
cargo run -p hello-world

# Request binding with validation
cargo run -p validation
```

## Project Status

zenix is in early development. The API is minimal by design and subject to change. Contributions and feedback are welcome!
