#[derive(Clone)]
pub struct AppState {
    pub version: String,
    pub http: reqwest::Client,
    /// Base URL de HN API — sobreescribible en tests con mockito
    pub hn_base_url: String,
}

impl AppState {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            http: reqwest::Client::new(),
            hn_base_url: "https://hacker-news.firebaseio.com/v0".to_string(),
        }
    }
}
