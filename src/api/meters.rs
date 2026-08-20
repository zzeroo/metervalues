use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    error::AppError,
    models::{CreateMeterInstance, Meter, MeterInstance, RemoveMeterInstance},
    state::AppState,
};

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

pub async fn create_meter_instance(
    State(state): State<AppState>,
    Path(meter_id): Path<i64>,
    Json(payload): Json<CreateMeterInstance>,
) -> Result<(StatusCode, Json<MeterInstance>), AppError> {
    let meter_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM meters
            WHERE id = $1
        )
        "#,
    )
    .bind(meter_id)
    .fetch_one(&state.db)
    .await?;

    if !meter_exists {
        return Err(AppError::NotFound);
    }

    let meter_instance = sqlx::query_as::<_, MeterInstance>(
        r#"
        INSERT INTO meter_instances (
            meter_id,
            meter_number,
            initial_reading,
            initial_reading_date,
            installed_at
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            meter_id,
            meter_number,
            initial_reading,
            initial_reading_date,
            installed_at,
            removed_at
        "#,
    )
    .bind(meter_id)
    .bind(payload.meter_number)
    .bind(payload.initial_reading)
    .bind(payload.initial_reading_date)
    .bind(payload.installed_at)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(meter_instance)))
}

pub async fn get_meter_instances(
    State(state): State<AppState>,
    Path(meter_id): Path<i64>,
) -> Result<Json<Vec<MeterInstance>>, AppError> {
    let instances = sqlx::query_as::<_, MeterInstance>(
        r#"
        SELECT
            id,
            meter_id,
            meter_number,
            initial_reading,
            initial_reading_date,
            installed_at,
            removed_at
        FROM meter_instances
        WHERE meter_id = $1
        ORDER BY installed_at
        "#,
    )
    .bind(meter_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(instances))
}

pub async fn remove_meter_instance(
    State(state): State<AppState>,
    Path(meter_instance_id): Path<i64>,
    Json(request): Json<RemoveMeterInstance>,
) -> Result<Json<MeterInstance>, AppError> {
    let meter_instance = sqlx::query_as::<_, MeterInstance>(
        r#"
        UPDATE meter_instances
        SET removed_at = $1
        WHERE id = $2
        RETURNING
            id,
            meter_id,
            meter_number,
            initial_reading,
            initial_reading_date,
            installed_at,
            removed_at,
            created_at
        "#,
    )
    .bind(request.removed_at)
    .bind(meter_instance_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(meter_instance))
}
