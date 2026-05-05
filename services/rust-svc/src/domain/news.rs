use reqwest::Client;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::error::AppError;

const HN_BASE: &str = "https://hacker-news.firebaseio.com/v0";

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

pub async fn fetch_top_news(client: &Client, n: usize) -> Result<Vec<NewsItem>, AppError> {
    let ids: Vec<u32> = client
        .get(format!("{HN_BASE}/topstories.json"))
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut items = Vec::with_capacity(n);
    for id in ids.into_iter().take(n) {
        let hn: HnItem = client
            .get(format!("{HN_BASE}/item/{id}.json"))
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
