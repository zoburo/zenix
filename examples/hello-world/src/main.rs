use zenix::{Context, Server};

#[tokio::main]
async fn main() {
    let mut server = Server::new();

    server.get("/", |ctx: Context| ctx.string("Hello, World!"));

    server.listen(":3000").await.unwrap();
}
