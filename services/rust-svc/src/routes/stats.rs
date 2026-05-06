// ❌ BOX-001: returns raw struct without { success, data } envelope
// ❌ BOX-002: uses .unwrap() in production code
// ❌ BOX-003: business logic (filtering, calculation) inline in handler
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::sync::Arc;
use crate::state::AppState;

#[derive(Serialize)]
pub struct StatsResponse {
    pub total: usize,
    pub avg_score: f64,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/stats", get(stats_handler))
}

// ❌ BOX-001: returns StatsResponse directly, not { "success": true, "data": ... }
// ❌ BOX-003: score calculation is business logic — should live in domain/stats.rs
async fn stats_handler() -> Json<StatsResponse> {
    // ❌ BOX-002: .unwrap() in production — will panic if file is missing or malformed
    let raw = std::fs::read_to_string("data.json").unwrap();
    let scores: Vec<u32> = serde_json::from_str(&raw).unwrap();

    // ❌ BOX-003: inline business logic — filtering and average belong in domain/
    let valid: Vec<u32> = scores.into_iter().filter(|&s| s > 0).collect();
    let avg = if valid.is_empty() {
        0.0
    } else {
        valid.iter().sum::<u32>() as f64 / valid.len() as f64
    };

    Json(StatsResponse {
        total: valid.len(),
        avg_score: avg,
    })
}
