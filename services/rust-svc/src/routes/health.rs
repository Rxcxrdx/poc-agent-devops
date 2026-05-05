use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_handler))
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Servicio operativo"))
)]
async fn health_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "ok",
            "version": state.version
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    fn test_server() -> TestServer {
        let state = Arc::new(AppState {
            version: "test".into(),
            http: reqwest::Client::new(),
        });
        TestServer::new(router().with_state(state))
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let res = test_server().get("/health").await;
        res.assert_status_ok();
    }

    #[tokio::test]
    async fn health_returns_envelope() {
        let res = test_server().get("/health").await;
        let body: serde_json::Value = res.json();
        assert_eq!(body["success"], true);
        assert_eq!(body["data"]["status"], "ok");
    }
}
