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

#[tokio::test]
async fn create_meter_instance() {
    let db = common::test_db().await;

    let app = metervalues::create_app(db.clone());

    let request_body = serde_json::json!({
        "meter_number": "TEST-1001",
        "initial_reading": "15234.000",
        "initial_reading_date": "2026-08-15",
        "installed_at": "2026-08-15"
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method(axum::http::Method::POST)
                .uri("/api/meters/1/instances")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_vec(&request_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["meter_id"], 1);
    assert_eq!(json["meter_number"], "TEST-1001");
    assert_eq!(json["initial_reading"], "15234.000");
    assert_eq!(json["initial_reading_date"], "2026-08-15");
    assert_eq!(json["installed_at"], "2026-08-15");
    assert!(json["removed_at"].is_null());

    // Cleanup
    sqlx::query("DELETE FROM meter_instances WHERE meter_number = $1")
        .bind("TEST-1001")
        .execute(&db)
        .await
        .expect("Could not clean up test instance");
}

#[tokio::test]
async fn create_meter_instance_for_nonexistent_meter_returns_not_found() {
    let db = common::test_db().await;

    let app = metervalues::create_app(db);

    let request_body = serde_json::json!({
        "meter_number": "TEST-NOT-FOUND",
        "initial_reading": "0.000",
        "initial_reading_date": "2026-08-15",
        "installed_at": "2026-08-15"
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method(axum::http::Method::POST)
                .uri("/api/meters/999999/instances")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_vec(&request_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["error"], "not_found");
}
