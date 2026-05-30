# zenix

High-performance and ergonomic Rust HTTP framework inspired by [Go Fiber](https://gofiber.io/).

## Quickstart

```rust
use zenix::{Context, Server};

#[tokio::main]
async fn main() {
    let mut server = Server::new();

    server.get("/", |ctx: Context| ctx.string("Hello, World!"));

    server.listen(":3000").await.unwrap();
}
```

## Features

### Routing

Register handlers for GET, POST, PUT, and DELETE:

```rust
server.get("/", handler);
server.post("/users", handler);
server.put("/users/:id", handler);
server.delete("/users/:id", handler);
```

### Request binding (`#[derive(FromRequest)]`)

Deserialize request data into structs with per-field source annotations:

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

| Annotation                 | Source            |
| -------------------------- | ----------------- |
| `#[zenix(from = "body")]`  | JSON request body |
| `#[zenix(from = "query")]` | URL query string  |
| `#[zenix(from = "form")]`  | URL-encoded body  |

### Validation

Integrates with [`garde`](https://docs.rs/garde) for validation:

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

### Responses

```rust
ctx.string("plain text");                          // 200 text/plain
ctx.json(r#"{"key":"value"}"#);                    // 200 application/json
ctx.html("<h1>title</h1>");                        // 200 text/html
ctx.status(StatusCode::CREATED).json(body);        // custom status
```

## Examples

```shell
cargo run -p hello-world
```

## Crate overview

| Crate          | Description                 |
| -------------- | --------------------------- |
| `zenix`        | Main framework (this crate) |
| `zenix-derive` | Proc-macros (re-exported)   |
