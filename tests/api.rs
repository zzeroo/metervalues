mod common;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header::CONTENT_TYPE},
};
use common::{cleanup_meter, test_db};
use rust_decimal::Decimal;
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
    cleanup_meter(&db, meter_id).await;
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
    cleanup_meter(&db, meter_id).await;
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
    cleanup_meter(&db, meter_id).await;
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
    cleanup_meter(&db, meter_id).await;
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

#[tokio::test]
async fn get_readings_returns_readings_in_chronological_order() {
    let db = test_db().await;

    // Create a dedicated logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Reading Order Test")
    .bind("m³")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    // Create a dedicated meter instance.
    let meter_instance_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meter_instances (
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
    .bind("TEST-READING-ORDER-1001")
    .bind(Decimal::new(0, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&db)
    .await
    .expect("Could not create test meter instance");

    // Insert readings deliberately out of chronological order.
    sqlx::query(
        r#"
        INSERT INTO readings (
            meter_instance_id,
            reading_date,
            value
        )
        VALUES
            ($1, $2, $3),
            ($1, $4, $5),
            ($1, $6, $7)
        "#,
    )
    .bind(meter_instance_id)
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 8, 20).unwrap())
    .bind(Decimal::new(300_000, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 8, 10).unwrap())
    .bind(Decimal::new(100_000, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .bind(Decimal::new(200_000, 3))
    .execute(&db)
    .await
    .expect("Could not create test readings");

    let app = metervalues::create_app(db.clone());

    let uri = format!("/api/meter-instances/{meter_instance_id}/readings");

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

    assert_eq!(json.as_array().unwrap().len(), 3);

    // The response must be sorted by reading_date ascending.
    assert_eq!(json[0]["reading_date"], "2026-08-10");
    assert_eq!(json[0]["value"], "100.000");

    assert_eq!(json[1]["reading_date"], "2026-08-15");
    assert_eq!(json[1]["value"], "200.000");

    assert_eq!(json[2]["reading_date"], "2026-08-20");
    assert_eq!(json[2]["value"], "300.000");

    // Cleanup: readings -> meter instances -> meters.
    cleanup_meter(&db, meter_id).await;
}

#[tokio::test]
async fn remove_meter_instance() {
    let db = test_db().await;

    // Create a dedicated logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Remove Meter Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    // Create an active meter instance.
    let meter_instance_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meter_instances (
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
    .bind("TEST-REMOVE-1001")
    .bind(rust_decimal::Decimal::new(1000, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&db)
    .await
    .expect("Could not create test meter instance");

    let app = metervalues::create_app(db.clone());

    let request_body = serde_json::json!({
        "removed_at": "2026-08-20"
    });

    let uri = format!("/api/meter-instances/{meter_instance_id}");

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(&uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
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

    assert_eq!(json["id"], meter_instance_id);
    assert_eq!(json["removed_at"], "2026-08-20");

    cleanup_meter(&db, meter_id).await;
}

#[tokio::test]
async fn remove_nonexistent_meter_instance_returns_not_found() {
    let db = test_db().await;

    let app = metervalues::create_app(db);

    let request_body = serde_json::json!({
        "removed_at": "2026-08-20"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/meter-instances/999999")
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
async fn exchange_meter_instance() {
    let db = test_db().await;

    // Create a dedicated logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Exchange Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    // Create the currently active meter instance.
    let old_meter_instance_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meter_instances (
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
    .bind("TEST-EXCHANGE-OLD")
    .bind(rust_decimal::Decimal::new(12345, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&db)
    .await
    .expect("Could not create old meter instance");

    let app = metervalues::create_app(db.clone());

    let request_body = serde_json::json!({
        "removed_at": "2026-08-20",
        "meter_number": "TEST-EXCHANGE-NEW",
        "initial_reading": "0.000",
        "initial_reading_date": "2026-08-20",
        "installed_at": "2026-08-20"
    });

    let uri = format!("/api/meter-instances/{old_meter_instance_id}/exchange");

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

    // The response should contain the newly created meter instance.
    assert_eq!(json["meter_id"], meter_id);
    assert_eq!(json["meter_number"], "TEST-EXCHANGE-NEW");
    assert_eq!(json["initial_reading"], "0");
    assert_eq!(json["installed_at"], "2026-08-20");
    assert!(json["removed_at"].is_null());

    // Verify that the old instance was actually removed.
    let removed_at: Option<chrono::NaiveDate> = sqlx::query_scalar(
        r#"
        SELECT removed_at
        FROM meter_instances
        WHERE id = $1
        "#,
    )
    .bind(old_meter_instance_id)
    .fetch_one(&db)
    .await
    .expect("Could not query old meter instance");

    assert_eq!(
        removed_at,
        Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 20).unwrap())
    );

    cleanup_meter(&db, meter_id).await;
}

#[tokio::test]
async fn failed_meter_exchange_keeps_old_instance_active() {
    let db = test_db().await;

    // Create a dedicated logical meter for the exchange.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Exchange Rollback Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    // Create the active meter instance we will attempt to exchange.
    let old_meter_instance_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meter_instances (
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
    .bind("TEST-EXCHANGE-ROLLBACK-OLD")
    .bind(rust_decimal::Decimal::new(1000, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&db)
    .await
    .expect("Could not create old meter instance");

    // Create another meter with a meter number that we will deliberately
    // try to reuse during the exchange.
    let duplicate_meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Duplicate Meter Number Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create duplicate test meter");

    sqlx::query(
        r#"
        INSERT INTO meter_instances (
            meter_id,
            meter_number,
            initial_reading,
            initial_reading_date,
            installed_at
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(duplicate_meter_id)
    .bind("TEST-EXCHANGE-DUPLICATE")
    .bind(rust_decimal::Decimal::new(0, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .execute(&db)
    .await
    .expect("Could not create duplicate meter instance");

    let app = metervalues::create_app(db.clone());

    let request_body = serde_json::json!({
        "removed_at": "2026-08-20",
        "meter_number": "TEST-EXCHANGE-DUPLICATE",
        "initial_reading": "0.000",
        "initial_reading_date": "2026-08-20",
        "installed_at": "2026-08-20"
    });

    let uri = format!("/api/meter-instances/{old_meter_instance_id}/exchange");

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

    // The duplicate meter number should cause a conflict.
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // Verify the API error response.
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Could not read response body");

    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["error"], "conflict");

    // Most importantly: verify that the old instance is still active.
    let removed_at: Option<chrono::NaiveDate> = sqlx::query_scalar(
        r#"
        SELECT removed_at
        FROM meter_instances
        WHERE id = $1
        "#,
    )
    .bind(old_meter_instance_id)
    .fetch_one(&db)
    .await
    .expect("Could not query old meter instance");

    assert!(
        removed_at.is_none(),
        "Old meter instance was removed even though the exchange failed"
    );

    // Cleanup.
    cleanup_meter(&db, meter_id).await;
    cleanup_meter(&db, duplicate_meter_id).await;
}

#[tokio::test]
async fn create_meter() {
    let db = test_db().await;

    let app = metervalues::create_app(db.clone());

    let request_body = serde_json::json!({
        "name": "API Test Water",
        "unit": "m³"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/meters")
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

    let meter_id = json["id"]
        .as_i64()
        .expect("Response does not contain a valid meter id");

    assert_eq!(json["name"], "API Test Water");
    assert_eq!(json["unit"], "m³");

    cleanup_meter(&db, meter_id).await;
}

#[tokio::test]
async fn get_meter_by_id() {
    let db = test_db().await;

    // Create a dedicated meter for this test.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Get Meter Test")
    .bind("m³")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let app = metervalues::create_app(db.clone());

    let uri = format!("/api/meters/{meter_id}");

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

    assert_eq!(json["id"], meter_id);
    assert_eq!(json["name"], "API Get Meter Test");
    assert_eq!(json["unit"], "m³");

    cleanup_meter(&db, meter_id).await;
}

#[tokio::test]
async fn get_nonexistent_meter_returns_not_found() {
    let db = test_db().await;

    let app = metervalues::create_app(db);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/meters/999999999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn import_meters_from_csv() {
    let db = test_db().await;
    let app = metervalues::create_app(db.clone());

    let csv_data = r#"name,unit
API Import Electricity,kWh
API Import Water,m³
"#;

    let request = Request::builder()
        .method("POST")
        .uri("/api/import/meters")
        .header("content-type", "text/csv")
        .body(Body::from(csv_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn import_meter_instances_from_csv() {
    let db = test_db().await;
    let app = metervalues::create_app(db.clone());

    // Create the logical meter referenced by the CSV.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("API Import Instance Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let csv_data = r#"meter_name,meter_number,initial_reading,initial_reading_date,installed_at,removed_at
API Import Instance Test,API-IMPORT-001,0.000,2026-01-01,2026-01-01,
"#;

    let request = Request::builder()
        .method("POST")
        .uri("/api/import/meter-instances")
        .header("content-type", "text/csv")
        .body(Body::from(csv_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    cleanup_meter(&db, meter_id).await;
}

#[tokio::test]
async fn import_readings_from_csv() {
    let db = test_db().await;
    let app = metervalues::create_app(db.clone());

    // Create a logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Reading Import API Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    // Create the physical meter instance referenced by the CSV.
    let meter_instance_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meter_instances (
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
    .bind("READING-IMPORT-001")
    .bind(rust_decimal::Decimal::new(0, 3))
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&db)
    .await
    .expect("Could not create test meter instance");

    let csv_data = r#"meter_number,reading_date,value
READING-IMPORT-001,2026-02-01,123.456
"#;

    let request = Request::builder()
        .method("POST")
        .uri("/api/import/readings")
        .header("content-type", "text/csv")
        .body(Body::from(csv_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let reading_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM readings
            WHERE meter_instance_id = $1
              AND reading_date = $2
              AND value = $3
        )
        "#,
    )
    .bind(meter_instance_id)
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap())
    .bind(rust_decimal::Decimal::new(123_456, 3))
    .fetch_one(&db)
    .await
    .expect("Could not query imported reading");

    assert!(reading_exists);

    cleanup_meter(&db, meter_id).await;
}
