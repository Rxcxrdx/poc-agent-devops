#[derive(Clone)]
pub struct AppState {
    pub version: String,
    pub http: reqwest::Client,
}
