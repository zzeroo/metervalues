use axum::{Json, extract::State};

use crate::{error::AppError, models::Meter, state::AppState};

pub async fn get_meters(State(state): State<AppState>) -> Result<Json<Vec<Meter>>, AppError> {
    let meters = sqlx::query_as::<_, Meter>(
        r#"
        SELECT id, name, unit
        FROM meters
        ORDER BY id
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(meters))
}
