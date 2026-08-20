use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateReading {
    pub reading_date: NaiveDate,
    pub value: Decimal,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReadingResponse {
    pub id: i64,
    pub meter_instance_id: i64,
    pub reading_date: NaiveDate,
    pub value: Decimal,
}

pub async fn create_reading(
    State(state): State<AppState>,
    Path(meter_instance_id): Path<i64>,
    Json(request): Json<CreateReading>,
) -> Result<(StatusCode, Json<ReadingResponse>), AppError> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM meter_instances
            WHERE id = $1
        )
        "#,
    )
    .bind(meter_instance_id)
    .fetch_one(&state.db)
    .await?;

    if !exists {
        return Err(AppError::NotFound);
    }

    let reading = sqlx::query_as::<_, ReadingResponse>(
        r#"
        INSERT INTO readings (
            meter_instance_id,
            reading_date,
            value
        )
        VALUES ($1, $2, $3)
        RETURNING
            id,
            meter_instance_id,
            reading_date,
            value
        "#,
    )
    .bind(meter_instance_id)
    .bind(request.reading_date)
    .bind(request.value)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(reading)))
}

pub async fn get_readings(
    State(state): State<AppState>,
    Path(meter_instance_id): Path<i64>,
) -> Result<Json<Vec<ReadingResponse>>, AppError> {
    let readings = sqlx::query_as::<_, ReadingResponse>(
        r#"
        SELECT
            id,
            meter_instance_id,
            reading_date,
            value
        FROM readings
        WHERE meter_instance_id = $1
        ORDER BY reading_date ASC
        "#,
    )
    .bind(meter_instance_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(readings))
}
