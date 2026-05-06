use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;
use crate::{domain::news::NewsItem, error::AppError, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/news", get(news_handler))
        .route("/api/v1/news/devops", get(devops_news_handler))
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
    let news = crate::domain::news::fetch_top_news(&state.http, 3, &state.hn_base_url).await?;
    Ok(Json(json!({ "success": true, "data": news })))
}

#[utoipa::path(
    get,
    path = "/api/v1/news/devops",
    responses(
        (status = 200, body = Vec<NewsItem>, description = "Noticias de Hacker News filtradas por DevOps"),
        (status = 500, description = "Error al contactar HN API")
    )
)]
pub async fn devops_news_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let news = crate::domain::news::fetch_devops_news(&state.http, 20, &state.hn_base_url).await?;
    Ok(Json(json!({ "success": true, "data": news })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use mockito::Server;

    async fn test_server_with_hn(hn_base_url: String) -> TestServer {
        let state = Arc::new(AppState {
            version: "test".into(),
            http: reqwest::Client::new(),
            hn_base_url,
        });
        TestServer::new(router().with_state(state))
    }

    #[tokio::test]
    async fn news_returns_200_with_envelope() {
        let mut hn = Server::new_async().await;
        let _m1 = hn.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[1]")
            .create_async().await;
        let _m2 = hn.mock("GET", "/item/1.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":1,"title":"Test","by":"user","score":5}"#)
            .create_async().await;

        let server = test_server_with_hn(hn.url()).await;
        let res = server.get("/api/v1/news").await;

        res.assert_status_ok();
        let body: serde_json::Value = res.json();
        assert_eq!(body["success"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    async fn news_returns_items_with_expected_fields() {
        let mut hn = Server::new_async().await;
        let _m1 = hn.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[42]")
            .create_async().await;
        let _m2 = hn.mock("GET", "/item/42.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":42,"title":"Rust rocks","by":"ferris","score":100}"#)
            .create_async().await;

        let server = test_server_with_hn(hn.url()).await;
        let res = server.get("/api/v1/news").await;
        let body: serde_json::Value = res.json();
        let items = body["data"].as_array().unwrap();

        assert!(!items.is_empty());
        assert!(items[0]["title"].is_string());
        assert!(items[0]["author"].is_string());
        assert!(items[0]["score"].is_number());
    }

    #[tokio::test]
    async fn news_returns_500_when_hn_unreachable() {
        // Apunta a un puerto que no existe → reqwest falla → AppError::Internal → 500
        let server = test_server_with_hn("http://127.0.0.1:19999".to_string()).await;
        let res = server.get("/api/v1/news").await;

        res.assert_status_internal_server_error();
        let body: serde_json::Value = res.json();
        assert_eq!(body["success"], false);
    }

    // ── /api/v1/news/devops ─────────────────────────────────────────────────

    #[tokio::test]
    async fn devops_news_returns_200_with_envelope() {
        let mut hn = Server::new_async().await;
        let _m1 = hn.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[501]")
            .create_async().await;
        let _m2 = hn.mock("GET", "/item/501.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":501,"title":"Kubernetes 2.0 ships","by":"ops","score":99}"#)
            .create_async().await;

        let server = test_server_with_hn(hn.url()).await;
        let res = server.get("/api/v1/news/devops").await;

        res.assert_status_ok();
        let body: serde_json::Value = res.json();
        assert_eq!(body["success"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    async fn devops_news_filters_non_devops_items() {
        let mut hn = Server::new_async().await;
        let _m1 = hn.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[601, 602]")
            .create_async().await;
        let _m2 = hn.mock("GET", "/item/601.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":601,"title":"Ask HN: best recipes","by":"chef","score":3}"#)
            .create_async().await;
        let _m3 = hn.mock("GET", "/item/602.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":602,"title":"CI/CD with GitHub Actions","by":"dev","score":77}"#)
            .create_async().await;

        let server = test_server_with_hn(hn.url()).await;
        let res = server.get("/api/v1/news/devops").await;

        let body: serde_json::Value = res.json();
        let items = body["data"].as_array().expect("data is array");
        // Solo el item de CI/CD debe aparecer
        assert_eq!(items.len(), 1);
        assert!(items[0]["title"].as_str().unwrap().to_lowercase().contains("ci/cd"));
    }

    #[tokio::test]
    async fn devops_news_returns_500_when_hn_unreachable() {
        let server = test_server_with_hn("http://127.0.0.1:19999".to_string()).await;
        let res = server.get("/api/v1/news/devops").await;

        res.assert_status_internal_server_error();
        let body: serde_json::Value = res.json();
        assert_eq!(body["success"], false);
    }
}
