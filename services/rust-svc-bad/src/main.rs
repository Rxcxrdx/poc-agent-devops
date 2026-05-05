use axum::Router;

mod routes {
    pub mod news;
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(routes::news::router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap(); // ❌ BOX-002: .unwrap() en main

    axum::serve(listener, app).await.unwrap(); // ❌ BOX-002
}
