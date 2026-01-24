use axum::{Json, Router, http::StatusCode, response::{Html, IntoResponse, Response}, routing::get};
use core::{get_history, Record, HistoryError};

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(handler))
        .route("/history", get(get_history_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

// #[derive(Deserialize)]
// struct GetHistoryParams {

// }

pub struct ApiError(pub HistoryError);

impl From<HistoryError> for ApiError {
    fn from(err: HistoryError) -> Self {
        ApiError(err)
    }
}


impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            HistoryError::InvalidRange => StatusCode::BAD_REQUEST,
            HistoryError::NotFound => StatusCode::NOT_FOUND,
        };

        (status, self.0.to_string()).into_response()
    }
}



async fn get_history_handler() -> Result<impl IntoResponse, ApiError> {
    let history = get_history(0, 0).await?;
    Ok(Json(history))
}