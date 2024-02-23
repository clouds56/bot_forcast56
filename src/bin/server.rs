use axum::{routing::{get, Route}, Router};


async fn index() -> &'static str {
  "hello world"
}

#[tokio::main]
pub async fn main() {
  let app = Router::new()
    .route("/api", get(index));
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}
