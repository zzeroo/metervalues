use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    NotFound,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Database(error) => {
                error!("Database error: {:?}", error);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "internal_server_error",
                    }),
                )
                    .into_response()
            }
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: "not_found" }),
            )
                .into_response(),
        }
    }
}
