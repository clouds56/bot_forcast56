use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use tokio::sync::Mutex;

type ArcAppState = std::sync::Arc<tokio::sync::Mutex<AppState>>;

#[derive(Clone)]
struct AppState {
  hosts: Vec<HostInfo>,
}

impl AppState {
  fn update_host(&mut self, host: HostInfo) {
    for h in &mut self.hosts {
      if h.host == host.host {
        *h = host;
        return;
      }
    }
    self.hosts.push(host);
  }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
struct HostInfo {
  host: String,
  ip: String,
}

async fn show_ip(State(state): State<ArcAppState>) -> Json<Vec<HostInfo>> {
  state.lock().await.hosts.clone().into()
}

async fn record_ip(State(state): State<ArcAppState>, Json(data): Json<HostInfo>) -> &'static str {
  state.lock().await.update_host(data);
  "ok"
}

pub async fn serve() {
  let state = Arc::new(Mutex::new(AppState { hosts: vec![] }));
  let app = Router::new()
    .route("/ip", get(show_ip).post(record_ip))
    .with_state(state);
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
pub async fn main() {
  serve().await;
}

#[cfg(test)]
mod test {

  use super::*;

#[must_use]
struct AbortHandle(tokio::task::AbortHandle);

impl Drop for AbortHandle {
  fn drop(&mut self) {
    self.0.abort();
  }
}

fn test_serve() -> AbortHandle {
  AbortHandle(tokio::spawn(super::serve()).abort_handle())
}

#[tokio::test]
async fn test_server() {
  let _handle = test_serve();
  let client = reqwest::Client::new();

  client.post("http://localhost:3000/ip").json(&HostInfo {
    host: "test".to_string(),
    ip: "127.0.0.0".to_string(),
  }).send().await.unwrap();

  client.post("http://localhost:3000/ip").json(&HostInfo {
    host: "test2".to_string(),
    ip: "127.0.0.2".to_string(),
  }).send().await.unwrap();

  client.post("http://localhost:3000/ip").json(&HostInfo {
    host: "test".to_string(),
    ip: "127.0.0.1".to_string(),
  }).send().await.unwrap();

  let result: Vec<HostInfo> = client.get("http://localhost:3000/ip").send().await.unwrap()
    .json().await.unwrap();
  assert_eq!(result.len(), 2);
  assert_eq!(result[0].host, "test");
  assert_eq!(result[0].ip, "127.0.0.1");
  assert_eq!(result[1].host, "test2");
  assert_eq!(result[1].ip, "127.0.0.2");
}

}
