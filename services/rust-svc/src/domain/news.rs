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

/// Keywords (minúsculas) que identifican noticias relacionadas con DevOps.
const DEVOPS_KEYWORDS: &[&str] = &[
    "devops",
    "dev ops",
    "kubernetes",
    "k8s",
    "docker",
    "helm",
    "terraform",
    "ansible",
    "jenkins",
    "github actions",
    "gitlab ci",
    "ci/cd",
    "continuous delivery",
    "continuous integration",
    "infrastructure as code",
    "site reliability",
    " sre",
];

/// Retorna `true` si el título contiene al menos un keyword de DevOps.
pub fn is_devops(title: &str) -> bool {
    let lower = title.to_lowercase();
    DEVOPS_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

/// Escanea hasta 200 top stories de HN y retorna las primeras `limit` que
/// mencionen DevOps en el título.
pub async fn fetch_devops_news(
    client: &Client,
    limit: usize,
    base_url: &str,
) -> Result<Vec<NewsItem>, AppError> {
    let ids: Vec<u32> = client
        .get(format!("{base_url}/topstories.json"))
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut results: Vec<NewsItem> = Vec::new();
    let scan_limit = ids.len().min(200);

    for id in ids.into_iter().take(scan_limit) {
        if results.len() >= limit {
            break;
        }
        let hn: HnItem = client
            .get(format!("{base_url}/item/{id}.json"))
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .json()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let title = hn.title.unwrap_or_default();
        if is_devops(&title) {
            results.push(NewsItem {
                id: hn.id,
                title,
                author: hn.by.unwrap_or_default(),
                url: hn.url,
                score: hn.score.unwrap_or(0),
            });
        }
    }

    tracing::info!(found = results.len(), "devops news fetched");
    Ok(results)
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

    // ── is_devops ──────────────────────────────────────────────────────────

    #[test]
    fn is_devops_matches_keyword() {
        assert!(is_devops("How Kubernetes changed our pipeline"));
        assert!(is_devops("DevOps for beginners"));
        assert!(is_devops("CI/CD best practices in 2025"));
        assert!(is_devops("Terraform 2.0 released"));
    }

    #[test]
    fn is_devops_case_insensitive() {
        assert!(is_devops("KUBERNETES is here"));
        assert!(is_devops("Docker Tips & Tricks"));
    }

    #[test]
    fn is_devops_rejects_unrelated() {
        assert!(!is_devops("New JavaScript framework released"));
        assert!(!is_devops("Python 4.0 roadmap"));
    }

    // ── fetch_devops_news ──────────────────────────────────────────────────

    #[tokio::test]
    async fn fetch_devops_returns_only_matching_items() {
        let mut server = Server::new_async().await;
        let _m1 = server.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[201, 202, 203]")
            .create_async().await;
        let _m2 = server.mock("GET", "/item/201.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":201,"title":"Kubernetes 1.30 released","by":"ops","score":80}"#)
            .create_async().await;
        let _m3 = server.mock("GET", "/item/202.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":202,"title":"A new JS framework","by":"web","score":10}"#)
            .create_async().await;
        let _m4 = server.mock("GET", "/item/203.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":203,"title":"CI/CD pipelines with GitHub Actions","by":"dev","score":55}"#)
            .create_async().await;

        let client = reqwest::Client::new();
        let result = fetch_devops_news(&client, 10, &server.url()).await.unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|i| is_devops(&i.title)));
    }

    #[tokio::test]
    async fn fetch_devops_respects_limit() {
        let mut server = Server::new_async().await;
        let _m1 = server.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[301, 302, 303]")
            .create_async().await;
        for (id, title) in [(301, "DevOps 1"), (302, "DevOps 2"), (303, "DevOps 3")] {
            server.mock("GET", &format!("/item/{id}.json")[..])
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(format!(r#"{{"id":{id},"title":"{title}","by":"u","score":1}}"#))
                .create_async().await;
        }

        let client = reqwest::Client::new();
        let result = fetch_devops_news(&client, 2, &server.url()).await.unwrap();

        assert!(result.len() <= 2);
    }

    #[tokio::test]
    async fn fetch_devops_returns_empty_when_no_match() {
        let mut server = Server::new_async().await;
        let _m = server.mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[401]")
            .create_async().await;
        let _m2 = server.mock("GET", "/item/401.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":401,"title":"New Python release","by":"py","score":5}"#)
            .create_async().await;

        let client = reqwest::Client::new();
        let result = fetch_devops_news(&client, 10, &server.url()).await.unwrap();

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
