use garde::Validate;
use hyper::StatusCode;
use serde::Deserialize;
use zenix::validate::ReportExt;
use zenix::{Context, FromRequest, Response, Server};

#[derive(Debug, Deserialize, Validate, FromRequest)]
struct CreateUserInput {
    /// Pulled from the JSON request body.
    #[zenix(from = "body")]
    #[garde(length(min = 1, max = 100))]
    name: String,

    #[zenix(from = "body")]
    #[garde(email)]
    email: String,

    #[zenix(from = "body")]
    #[garde(range(min = 18, max = 150))]
    age: u8,

    #[zenix(from = "body")]
    #[garde(length(min = 10, max = 500))]
    bio: String,
}

fn handle_create_user(ctx: Context) -> Response {
    // Deserialise from the JSON body using the #[zenix(from)] annotations
    let input: CreateUserInput = match ctx.bind() {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if let Err(report) = input.validate() {
        let body = report.to_json();
        return ctx.status(StatusCode::UNPROCESSABLE_ENTITY).json(body);
    }

    let body = format!(
        r#"{{"created":"{}","email":"{}","age":{}}}"#,
        input.name, input.email, input.age
    );

    ctx.status(StatusCode::CREATED).json(body)
}

#[tokio::main]
async fn main() {
    let mut server = Server::new();

    server.post("/users", handle_create_user);

    server.listen(":3000").await.unwrap();
}
