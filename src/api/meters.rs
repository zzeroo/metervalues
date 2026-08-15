use axum::{Json, extract::State, http::StatusCode};

use crate::{models::Meter, state::AppState};

pub async fn get_meters(State(state): State<AppState>) -> Result<Json<Vec<Meter>>, StatusCode> {
    let meters = sqlx::query_as::<_, Meter>(
        r#"
        SELECT id, name, unit
        FROM meters
        ORDER BY id
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(meters))
}
