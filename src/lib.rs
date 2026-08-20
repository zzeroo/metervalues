use axum::Router;
use sqlx::PgPool;

mod api;
mod error;
pub mod import;
mod models;
mod state;

pub fn create_app(db: PgPool) -> Router {
    let state = state::AppState { db };

    api::router().with_state(state)
}
