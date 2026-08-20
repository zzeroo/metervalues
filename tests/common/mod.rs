use sqlx::PgPool;

pub async fn test_db() -> PgPool {
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

#[allow(dead_code)]
pub async fn cleanup_meter(db: &PgPool, meter_id: i64) {
    sqlx::query(
        r#"
        DELETE FROM readings
        WHERE meter_instance_id IN (
            SELECT id
            FROM meter_instances
            WHERE meter_id = $1
        )
        "#,
    )
    .bind(meter_id)
    .execute(db)
    .await
    .expect("Could not clean up test readings");

    sqlx::query("DELETE FROM meter_instances WHERE meter_id = $1")
        .bind(meter_id)
        .execute(db)
        .await
        .expect("Could not clean up test meter instances");

    sqlx::query("DELETE FROM meters WHERE id = $1")
        .bind(meter_id)
        .execute(db)
        .await
        .expect("Could not clean up test meter");
}
