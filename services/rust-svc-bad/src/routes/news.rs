// ❌ BOX-001: no hay envelope { success, data }
// ❌ BOX-002: usa .unwrap() en código de producción
// ❌ BOX-003: lógica de negocio inline en el handler, no en domain/

use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct NewsItem {
    id: u32,
    title: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/news", get(news_handler))
}

// ❌ BOX-001: devuelve Vec directamente, sin { success, data }
// ❌ BOX-003: el filtrado es lógica de negocio, debería estar en domain/
async fn news_handler() -> Json<Vec<NewsItem>> {
    // ❌ BOX-002: .unwrap() en producción — puede causar panic
    let raw = std::fs::read_to_string("news.json").unwrap();
    let all: Vec<NewsItem> = serde_json::from_str(&raw).unwrap();

    // ❌ BOX-003: filtrado de negocio inline en el handler
    let filtered: Vec<NewsItem> = all.into_iter().filter(|n| n.id > 0).collect();
    Json(filtered)
}
