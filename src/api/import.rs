use axum::{body::Bytes, extract::State, http::StatusCode};

use crate::{
    import::{
        import_meter_instances as import_meter_instances_csv, import_meters as import_meters_csv,
    },
    state::AppState,
};

pub async fn import_meters(
    State(state): State<AppState>,
    csv_data: Bytes,
) -> Result<StatusCode, StatusCode> {
    import_meters_csv(&state.db, &csv_data)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

pub async fn import_meter_instances(
    State(state): State<AppState>,
    csv_data: Bytes,
) -> Result<StatusCode, StatusCode> {
    import_meter_instances_csv(&state.db, &csv_data)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|_| StatusCode::BAD_REQUEST)
}
