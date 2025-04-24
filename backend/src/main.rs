use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use state::AppState;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState::new(
        "uploads".to_string(),
        sqlx::PgPool::connect("postgres://postgres:postgres@localhost:5432/cloud")
            .await
            .unwrap(),
    ));

    let app = Router::new()
        .route("/api/v1/upload", post(routes::upload::handler))
        .route("/api/v1/download/{id}", get(routes::download::handler))
        .route("/api/v1/files", post(routes::files::get_handler))
        // .route("/api/v1/files/{id}", get(routes::files::get_handler).delete(routes::files::delete_handler))
        .route(
            "/api/v1/folder",
            post(routes::folder::create_folder)
                .delete(routes::folder::delete_folder)
                .put(routes::folder::rename_folder)
                .patch(routes::folder::move_folder),
        )
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
