use std::sync::Arc;
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod state;
mod error;
mod routes;
mod domain;

use state::AppState;
use routes::openapi::ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = Arc::new(AppState::new(env!("CARGO_PKG_VERSION")));

    // Rutas con estado — se resuelve el estado antes de unir SwaggerUi
    let api = Router::new()
        .merge(routes::health::router())
        .merge(routes::news::router())
        .merge(routes::openapi::router())
        .with_state(state);

    // SwaggerUi es stateless — usa /api-docs/openapi.json para no colisionar con /openapi.json
    let app = Router::new()
        .merge(api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::info!("rust-svc listening on 0.0.0.0:3000");
    tracing::info!("Swagger UI → http://localhost:3000/swagger-ui/");
    axum::serve(listener, app).await?;

    Ok(())
}
