use lambda_http::{run, service_fn, tracing};
use lambda_http::{Body, Error, Request, RequestExt, Response};
use core::{get_history};

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // extract query params if applicable
    
    let history = get_history(0, 0).await;

    let body = serde_json::to_string(&history)?;

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body.into())?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
