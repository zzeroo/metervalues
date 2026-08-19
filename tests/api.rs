mod common;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header::CONTENT_TYPE},
};
use common::test_db;
use tower::ServiceExt;

#[tokio::test]
async fn get_meters_returns_all_meters() {
    let db = test_db().await;

    let app = metervalues::create_app(db);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/meters")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_meter_instance() {
    let db = test_db().await;

    // Create a dedicated logical meter for this test.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Test Electricity")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let meter_number = format!("TEST-1001-{meter_id}");

    let app = metervalues::create_app(db.clone());

    let request_body = serde_json::json!({
        "meter_number": meter_number,
        "initial_reading": "15234.000",
        "initial_reading_date": "2026-08-15",
        "installed_at": "2026-08-15"
    });

    let uri = format!("/api/meters/{meter_id}/instances");

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["meter_id"], meter_id);
    assert_eq!(json["meter_number"], meter_number);
    assert_eq!(json["initial_reading"], "15234.000");
    assert_eq!(json["initial_reading_date"], "2026-08-15");
    assert_eq!(json["installed_at"], "2026-08-15");
    assert!(json["removed_at"].is_null());

    // Cleanup: child rows first because of the foreign key.
    sqlx::query("DELETE FROM meter_instances WHERE meter_id = $1")
        .bind(meter_id)
        .execute(&db)
        .await
        .expect("Could not clean up test meter instances");

    sqlx::query("DELETE FROM meters WHERE id = $1")
        .bind(meter_id)
        .execute(&db)
        .await
        .expect("Could not clean up test meter");
}

#[tokio::test]
async fn create_meter_instance_for_nonexistent_meter_returns_not_found() {
    let db = test_db().await;

    let app = metervalues::create_app(db);

    let request_body = serde_json::json!({
        "meter_number": "TEST-NOT-FOUND",
        "initial_reading": "0.000",
        "initial_reading_date": "2026-08-15",
        "installed_at": "2026-08-15"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/meters/999999/instances")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["error"], "not_found");
}

#[tokio::test]
async fn create_second_active_meter_instance_returns_conflict() {
    let db = test_db().await;

    // Create a dedicated logical meter for this test.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Conflict Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let first_meter_number = format!("TEST-CONFLICT-1001-{meter_id}");

    let second_meter_number = format!("TEST-CONFLICT-1002-{meter_id}");

    let app = metervalues::create_app(db.clone());

    let uri = format!("/api/meters/{meter_id}/instances");

    // Create the first active meter instance.
    let first_request_body = serde_json::json!({
        "meter_number": first_meter_number,
        "initial_reading": "1000.000",
        "initial_reading_date": "2026-08-15",
        "installed_at": "2026-08-15"
    });

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&first_request_body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("First request failed");

    assert_eq!(first_response.status(), StatusCode::CREATED);

    // Try to create a second active meter instance.
    let second_request_body = serde_json::json!({
        "meter_number": second_meter_number,
        "initial_reading": "0.000",
        "initial_reading_date": "2026-08-16",
        "installed_at": "2026-08-16"
    });

    let second_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&second_request_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("Second request failed");

    assert_eq!(second_response.status(), StatusCode::CONFLICT);

    let body = axum::body::to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["error"], "conflict");

    // Cleanup: child rows first, then the logical meter.
    sqlx::query("DELETE FROM meter_instances WHERE meter_id = $1")
        .bind(meter_id)
        .execute(&db)
        .await
        .expect("Could not clean up test meter instances");

    sqlx::query("DELETE FROM meters WHERE id = $1")
        .bind(meter_id)
        .execute(&db)
        .await
        .expect("Could not clean up test meter");
}
