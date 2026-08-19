use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use crate::state::AppState;

mod meters;
mod readings;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/meters", get(meters::get_meters))
        .route(
            "/api/meters/{meter_id}/instances",
            get(meters::get_meter_instances).post(meters::create_meter_instance),
        )
        .route(
            "/api/meter-instances/{meter_instance_id}/readings",
            post(readings::create_reading),
        )
        .route("/health", get(health))
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok"
    }))
}
