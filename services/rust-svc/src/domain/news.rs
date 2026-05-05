use reqwest::Client;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::error::AppError;

/// Historia de Hacker News
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct NewsItem {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub url: Option<String>,
    pub score: u32,
}

/// Shape interno de la HN API — no se expone al cliente
#[derive(Deserialize)]
struct HnItem {
    id: u32,
    title: Option<String>,
    by: Option<String>,
    url: Option<String>,
    score: Option<u32>,
}

pub async fn fetch_top_news(client: &Client, n: usize, base_url: &str) -> Result<Vec<NewsItem>, AppError> {
    let ids: Vec<u32> = client
        .get(format!("{base_url}/topstories.json"))
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut items = Vec::with_capacity(n);
    for id in ids.into_iter().take(n) {
        let hn: HnItem = client
            .get(format!("{base_url}/item/{id}.json"))
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        items.push(NewsItem {
            id: hn.id,
            title: hn.title.unwrap_or_default(),
            author: hn.by.unwrap_or_default(),
            url: hn.url,
            score: hn.score.unwrap_or(0),
        });
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn fetch_returns_n_items() {
        let mut server = Server::new_async().await;
        let _m1 = server.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[101, 102, 103, 104]")
            .create_async().await;
        let _m2 = server.mock("GET", "/item/101.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":101,"title":"Test A","by":"alice","score":42}"#)
            .create_async().await;
        let _m3 = server.mock("GET", "/item/102.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":102,"title":"Test B","by":"bob","score":10}"#)
            .create_async().await;

        let client = reqwest::Client::new();
        let result = fetch_top_news(&client, 2, &server.url()).await.unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn fetch_maps_fields_correctly() {
        let mut server = Server::new_async().await;
        let _m1 = server.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[99]")
            .create_async().await;
        let _m2 = server.mock("GET", "/item/99.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":99,"title":"Rust is great","by":"ferris","url":"https://rust-lang.org","score":999}"#)
            .create_async().await;

        let client = reqwest::Client::new();
        let result = fetch_top_news(&client, 1, &server.url()).await.unwrap();
        let item = &result[0];

        assert_eq!(item.id, 99);
        assert_eq!(item.title, "Rust is great");
        assert_eq!(item.author, "ferris");
        assert_eq!(item.score, 999);
        assert_eq!(item.url.as_deref(), Some("https://rust-lang.org"));
    }

    #[tokio::test]
    async fn fetch_handles_missing_optional_fields() {
        let mut server = Server::new_async().await;
        let _m1 = server.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[55]")
            .create_async().await;
        // title, by, url, score todos ausentes
        let _m2 = server.mock("GET", "/item/55.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":55}"#)
            .create_async().await;

        let client = reqwest::Client::new();
        let result = fetch_top_news(&client, 1, &server.url()).await.unwrap();
        let item = &result[0];

        assert_eq!(item.id, 55);
        assert_eq!(item.title, "");
        assert_eq!(item.author, "");
        assert_eq!(item.score, 0);
        assert!(item.url.is_none());
    }

    #[tokio::test]
    async fn fetch_returns_empty_when_no_stories() {
        let mut server = Server::new_async().await;
        let _m = server.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async().await;

        let client = reqwest::Client::new();
        let result = fetch_top_news(&client, 5, &server.url()).await.unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn news_item_serializes_correctly() {
        let item = NewsItem {
            id: 1,
            title: "Hello".to_string(),
            author: "world".to_string(),
            url: Some("https://example.com".to_string()),
            score: 100,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["title"], "Hello");
        assert_eq!(json["author"], "world");
        assert_eq!(json["score"], 100);
    }
}
