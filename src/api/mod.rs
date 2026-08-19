use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use crate::state::AppState;

mod meters;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/meters", get(meters::get_meters))
        .route(
            "/api/meters/{id}/instances",
            post(meters::create_meter_instance),
        )
        .route("/health", get(health))
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok"
    }))
}
