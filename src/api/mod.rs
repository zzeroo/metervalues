use axum::{
    Router,
    routing::{get, patch, post},
};
use tower_http::services::ServeDir;

use crate::state::AppState;

mod import;
mod meters;
mod readings;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/meters",
            get(meters::get_meters).post(meters::create_meter),
        )
        .route("/api/meters/{id}", get(meters::get_meter))
        .route(
            "/api/meters/{meter_id}/instances",
            get(meters::get_meter_instances).post(meters::create_meter_instance),
        )
        .route(
            "/api/meter-instances/{meter_instance_id}/readings",
            get(readings::get_readings).post(readings::create_reading),
        )
        .route(
            "/api/meter-instances/{meter_instance_id}",
            patch(meters::remove_meter_instance),
        )
        .route(
            "/api/meter-instances/{old_meter_instance_id}/exchange",
            post(meters::exchange_meter_instance),
        )
        .route("/api/import/meters", post(import::import_meters))
        .route(
            "/api/import/meter-instances",
            post(import::import_meter_instances),
        )
        .route("/health", get(health))
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok"
    }))
}
