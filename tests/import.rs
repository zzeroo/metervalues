mod common;

use metervalues::import::import_meters;
use sqlx::PgPool;

use common::{cleanup_meter, test_db};

#[tokio::test]
async fn import_meters_from_valid_csv() {
    let db: PgPool = test_db().await;

    let csv_data = r#"name,unit
Import Test Electricity,kWh
Import Test Water,m³
"#;

    let imported_meter_ids = import_meters(&db, csv_data.as_bytes())
        .await
        .expect("Could not import meters");

    assert_eq!(imported_meter_ids.len(), 2);

    let electricity_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM meters
            WHERE name = $1
              AND unit = $2
        )
        "#,
    )
    .bind("Import Test Electricity")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not query imported electricity meter");

    assert!(electricity_exists);

    let water_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM meters
            WHERE name = $1
              AND unit = $2
        )
        "#,
    )
    .bind("Import Test Water")
    .bind("m³")
    .fetch_one(&db)
    .await
    .expect("Could not query imported water meter");

    assert!(water_exists);

    for meter_id in imported_meter_ids {
        cleanup_meter(&db, meter_id).await;
    }
}

#[tokio::test]
async fn import_meters_with_invalid_csv_returns_error() {
    let db = test_db().await;

    let csv_data = r#"name
Invalid Import Test
"#;

    let result = import_meters(&db, csv_data.as_bytes()).await;

    assert!(result.is_err());

    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM meters
            WHERE name = $1
        )
        "#,
    )
    .bind("Invalid Import Test")
    .fetch_one(&db)
    .await
    .expect("Could not query test database");

    assert!(!exists);
}

#[tokio::test]
async fn import_meters_rolls_back_when_csv_contains_invalid_row() {
    let db = test_db().await;

    let csv_data = r#"name,unit
Import Rollback Test,kWh
Invalid Import Test
"#;

    let result = import_meters(&db, csv_data.as_bytes()).await;

    assert!(result.is_err());

    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM meters
            WHERE name = $1
        )
        "#,
    )
    .bind("Import Rollback Test")
    .fetch_one(&db)
    .await
    .expect("Could not query test database");

    assert!(
        !exists,
        "A valid row was imported even though the complete CSV import failed"
    );
}
