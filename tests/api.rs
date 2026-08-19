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

#[tokio::test]
async fn get_meter_instances_returns_all_instances() {
    let db = test_db().await;

    // Create a dedicated logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Instances Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let first_meter_number = format!("TEST-INSTANCE-1-{meter_id}");
    let second_meter_number = format!("TEST-INSTANCE-2-{meter_id}");

    // Insert first physical meter.
    sqlx::query(
        r#"
        INSERT INTO meter_instances
            (
                meter_id,
                meter_number,
                initial_reading,
                initial_reading_date,
                installed_at,
                removed_at
            )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(meter_id)
    .bind(&first_meter_number)
    .bind(rust_decimal::Decimal::new(1000, 0))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()))
    .execute(&db)
    .await
    .expect("Could not create first meter instance");

    // Insert replacement physical meter.
    sqlx::query(
        r#"
        INSERT INTO meter_instances
            (
                meter_id,
                meter_number,
                initial_reading,
                initial_reading_date,
                installed_at
            )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(meter_id)
    .bind(&second_meter_number)
    .bind(rust_decimal::Decimal::new(0, 0))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap())
    .execute(&db)
    .await
    .expect("Could not create second meter instance");

    let app = metervalues::create_app(db.clone());

    let uri = format!("/api/meters/{meter_id}/instances");

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json.as_array().unwrap().len(), 2);

    assert_eq!(json[0]["meter_id"], meter_id);
    assert_eq!(json[0]["meter_number"], first_meter_number);
    assert_eq!(json[0]["initial_reading"], "1000.000");
    assert_eq!(json[0]["removed_at"], "2026-06-01");

    assert_eq!(json[1]["meter_id"], meter_id);
    assert_eq!(json[1]["meter_number"], second_meter_number);
    assert_eq!(json[1]["initial_reading"], "0");
    assert_eq!(json[1]["removed_at"], serde_json::Value::Null);

    // Cleanup: child rows first.
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
async fn create_reading() {
    let db = test_db().await;

    // Create a dedicated logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Reading Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let meter_number = format!("TEST-READING-{meter_id}");

    // Create a dedicated physical meter instance.
    let meter_instance_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meter_instances
            (
                meter_id,
                meter_number,
                initial_reading,
                initial_reading_date,
                installed_at
            )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(meter_id)
    .bind(&meter_number)
    .bind(rust_decimal::Decimal::new(1000, 0))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&db)
    .await
    .expect("Could not create test meter instance");

    let app = metervalues::create_app(db.clone());

    let request_body = serde_json::json!({
        "reading_date": "2026-08-19",
        "value": "1234.567"
    });

    let uri = format!("/api/meter-instances/{meter_instance_id}/readings");

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request_body).expect("Could not serialize request body"),
                ))
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

    assert_eq!(json["meter_instance_id"], meter_instance_id);
    assert_eq!(json["reading_date"], "2026-08-19");
    assert_eq!(json["value"], "1234.567");

    // Cleanup: readings first, then meter instance, then logical meter.
    sqlx::query("DELETE FROM readings WHERE meter_instance_id = $1")
        .bind(meter_instance_id)
        .execute(&db)
        .await
        .expect("Could not clean up test readings");

    sqlx::query("DELETE FROM meter_instances WHERE id = $1")
        .bind(meter_instance_id)
        .execute(&db)
        .await
        .expect("Could not clean up test meter instance");

    sqlx::query("DELETE FROM meters WHERE id = $1")
        .bind(meter_id)
        .execute(&db)
        .await
        .expect("Could not clean up test meter");
}

#[tokio::test]
async fn create_reading_for_nonexistent_meter_instance_returns_not_found() {
    let db = test_db().await;

    let app = metervalues::create_app(db);

    let request_body = serde_json::json!({
        "reading_date": "2026-08-20",
        "value": "12345.000"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/meter-instances/999999/readings")
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
