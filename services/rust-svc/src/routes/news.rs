use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;
use crate::{domain::news::NewsItem, error::AppError, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/news", get(news_handler))
}

#[utoipa::path(
    get,
    path = "/api/v1/news",
    responses(
        (status = 200, body = Vec<NewsItem>, description = "Top 3 noticias de Hacker News"),
        (status = 500, description = "Error al contactar HN API")
    )
)]
pub async fn news_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let news = crate::domain::news::fetch_top_news(&state.http, 3).await?;
    Ok(Json(json!({ "success": true, "data": news })))
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
    async fn news_returns_ok() {
        let res = test_server().get("/api/v1/news").await;
        res.assert_status_ok();
    }

    #[tokio::test]
    async fn news_returns_envelope() {
        let res = test_server().get("/api/v1/news").await;
        let body: serde_json::Value = res.json();
        assert_eq!(body["success"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    async fn news_returns_items_with_fields() {
        let res = test_server().get("/api/v1/news").await;
        let body: serde_json::Value = res.json();
        let items = body["data"].as_array().expect("data debe ser array");
        assert!(!items.is_empty());
        assert!(items[0]["title"].is_string());
        assert!(items[0]["author"].is_string());
        assert!(items[0]["score"].is_number());
    }
}
