use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};

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

async fn test_transaction(db: &PgPool) -> Transaction<'_, Postgres> {
    db.begin().await.expect("Could not start test transaction")
}

#[tokio::test]
async fn only_one_active_meter_is_allowed() {
    let db = test_db().await;
    let mut tx = test_transaction(&db).await;

    // Create a logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Test Electricity")
    .bind("kWh")
    .fetch_one(&mut *tx)
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
    .bind(Decimal::new(1_000_000, 3))
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .execute(&mut *tx)
    .await
    .expect("First meter should be insertable");

    // Create a savepoint because the following statement is
    // intentionally expected to fail.
    sqlx::query("SAVEPOINT second_active_meter")
        .execute(&mut *tx)
        .await
        .expect("Could not create savepoint");

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
    .bind(Decimal::ZERO)
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_err(),
        "A second active meter should not be allowed"
    );

    // The failed INSERT aborted the transaction state.
    // Roll back only to the savepoint so the test can finish normally.
    sqlx::query("ROLLBACK TO SAVEPOINT second_active_meter")
        .execute(&mut *tx)
        .await
        .expect("Could not roll back to savepoint");

    // Everything created by this test will be removed by the
    // final transaction rollback.
    tx.rollback()
        .await
        .expect("Could not roll back test transaction");
}

#[tokio::test]
async fn meter_exchange_allows_new_active_meter() {
    let db = test_db().await;
    let mut tx = test_transaction(&db).await;

    // Create a logical meter.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Test Water")
    .bind("m³")
    .fetch_one(&mut *tx)
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
    .bind(Decimal::new(500_000, 3))
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .fetch_one(&mut *tx)
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
    .execute(&mut *tx)
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
    .bind(Decimal::ZERO)
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .bind(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_ok(),
        "A replacement meter should be allowed after the old meter is removed"
    );

    // No manual cleanup required.
    tx.rollback()
        .await
        .expect("Could not roll back test transaction");
}
