use chrono::NaiveDate;
use sqlx::PgPool;

async fn test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    assert!(
        database_url.contains("metervalues_test"),
        "Tests must run against metervalues_test"
    );

    let db = PgPool::connect(&database_url)
        .await
        .expect("Could not connect to test database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Could not run database migrations");

    db
}

#[tokio::test]
async fn only_one_active_meter_is_allowed() {
    let db = test_db().await;

    // Create a unique logical meter for this test.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Test Electricity")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    // First physical meter.
    sqlx::query(
        r#"
        INSERT INTO meter_instances
            (meter_id, meter_number, initial_reading,
             initial_reading_date, installed_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(meter_id)
    .bind("TEST-1001")
    .bind(1000.0_f64)
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .execute(&db)
    .await
    .expect("First meter should be insertable");

    // Second active meter must fail.
    let result = sqlx::query(
        r#"
        INSERT INTO meter_instances
            (meter_id, meter_number, initial_reading,
             initial_reading_date, installed_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(meter_id)
    .bind("TEST-1002")
    .bind(0.0_f64)
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .execute(&db)
    .await;

    assert!(
        result.is_err(),
        "A second active meter should not be allowed"
    );

    // Cleanup.
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
async fn meter_exchange_allows_new_active_meter() {
    let db = test_db().await;

    // Create a unique logical meter for this test.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Test Water")
    .bind("m³")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    // Old physical meter.
    let old_meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meter_instances
            (meter_id, meter_number, initial_reading,
             initial_reading_date, installed_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(meter_id)
    .bind("TEST-OLD-1001")
    .bind(500.0_f64)
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&db)
    .await
    .expect("Could not create old meter");

    // Simulate provider exchange.
    sqlx::query(
        r#"
        UPDATE meter_instances
        SET removed_at = $1
        WHERE id = $2
        "#,
    )
    .bind(NaiveDate::from_ymd_opt(2026, 8, 14).unwrap())
    .bind(old_meter_id)
    .execute(&db)
    .await
    .expect("Could not remove old meter");

    // New physical meter should now be allowed.
    let result = sqlx::query(
        r#"
        INSERT INTO meter_instances
            (meter_id, meter_number, initial_reading,
             initial_reading_date, installed_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(meter_id)
    .bind("TEST-NEW-1002")
    .bind(0.0_f64)
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .execute(&db)
    .await;

    assert!(
        result.is_ok(),
        "A replacement meter should be allowed after the old meter is removed"
    );

    // Cleanup.
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
