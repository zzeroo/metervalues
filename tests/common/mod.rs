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
