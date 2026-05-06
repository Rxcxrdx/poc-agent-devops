use axum::{response::IntoResponse, routing::get, Json, Router};
use std::sync::Arc;
use utoipa::OpenApi;
use crate::{domain::news::NewsItem, state::AppState};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health_handler,
        crate::routes::news::news_handler,
        crate::routes::news::devops_news_handler,
    ),
    components(schemas(NewsItem)),
    info(title = "rust-svc", version = "0.1.0", description = "API de ejemplo — PoC Conformance Gate")
)]
pub struct ApiDoc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/openapi.json", get(openapi_handler))
}

async fn openapi_handler() -> impl IntoResponse {
    match serde_json::to_value(ApiDoc::openapi()) {
        Ok(spec) => (axum::http::StatusCode::OK, Json(spec)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    fn test_server() -> TestServer {
        let state = Arc::new(AppState::new("test"));
        TestServer::new(router().with_state(state))
    }

    #[tokio::test]
    async fn openapi_returns_ok() {
        let res = test_server().get("/openapi.json").await;
        res.assert_status_ok();
    }

    #[tokio::test]
    async fn openapi_has_openapi_field() {
        let res = test_server().get("/openapi.json").await;
        let body: serde_json::Value = res.json();
        assert!(body["openapi"].is_string());
    }
}
