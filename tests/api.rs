mod common;

use common::test_db;

use metervalues::create_app;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn get_meters_returns_all_meters() {
    let db = test_db().await;

    let app = create_app(db);

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/meters")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: Value = serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(
        json,
        serde_json::json!([
            {
                "id": 1,
                "name": "Electricity",
                "unit": "kWh"
            },
            {
                "id": 2,
                "name": "Water",
                "unit": "m³"
            },
            {
                "id": 3,
                "name": "Gas",
                "unit": "m³"
            }
        ])
    );
}
