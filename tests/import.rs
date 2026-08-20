mod common;

use metervalues::import::{import_meter_instances, import_meters};

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

#[tokio::test]
async fn import_meter_instances_from_valid_csv() {
    let db = test_db().await;

    // Create the logical meter that the CSV will reference.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Import Electricity")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let csv_data = r#"meter_name,meter_number,initial_reading,initial_reading_date,installed_at
Import Electricity,IMPORT-12345,0.000,2026-01-01,2026-01-01
"#;

    let result = import_meter_instances(&db, csv_data.as_bytes()).await;

    assert!(
        result.is_ok(),
        "Expected meter instance import to succeed: {result:?}"
    );

    let imported_meter_instance_ids = result.unwrap();
    assert_eq!(imported_meter_instance_ids.len(), 1);
    let imported_meter_instance_id = imported_meter_instance_ids[0];

    let meter_instance: (
        i64,
        String,
        rust_decimal::Decimal,
        chrono::NaiveDate,
        chrono::NaiveDate,
    ) = sqlx::query_as(
        r#"
        SELECT
            meter_id,
            meter_number,
            initial_reading,
            initial_reading_date,
            installed_at
        FROM meter_instances
        WHERE id = $1
        "#,
    )
    .bind(imported_meter_instance_id)
    .fetch_one(&db)
    .await
    .expect("Could not find imported meter instance");

    assert_eq!(meter_instance.0, meter_id);
    assert_eq!(meter_instance.1, "IMPORT-12345");
    assert_eq!(meter_instance.2, rust_decimal::Decimal::new(0, 3));
    assert_eq!(
        meter_instance.3,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );
    assert_eq!(
        meter_instance.4,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    cleanup_meter(&db, meter_id).await;
}

#[tokio::test]
async fn import_multiple_meter_instances_from_valid_csv() {
    let db = test_db().await;

    // Create the logical meter referenced by both CSV rows.
    let meter_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO meters (name, unit)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Multiple Instance Import Test")
    .bind("kWh")
    .fetch_one(&db)
    .await
    .expect("Could not create test meter");

    let csv_data = r#"meter_name,meter_number,initial_reading,initial_reading_date,installed_at,removed_at
Multiple Instance Import Test,IMPORT-MULTI-001,0.000,2026-01-01,2026-01-01,2026-08-19
Multiple Instance Import Test,IMPORT-MULTI-002,100.000,2026-08-20,2026-08-20,
"#;

    let result = import_meter_instances(&db, csv_data.as_bytes()).await;

    assert!(
        result.is_ok(),
        "Expected meter instance import to succeed: {result:?}"
    );

    let imported_meter_instance_ids = result.unwrap();

    assert_eq!(
        imported_meter_instance_ids.len(),
        2,
        "Expected both meter instances to be imported"
    );

    let imported_instances: Vec<(i64, String, rust_decimal::Decimal)> = sqlx::query_as(
        r#"
        SELECT meter_id, meter_number, initial_reading
        FROM meter_instances
        WHERE id = ANY($1)
        ORDER BY meter_number
        "#,
    )
    .bind(&imported_meter_instance_ids)
    .fetch_all(&db)
    .await
    .expect("Could not query imported meter instances");

    assert_eq!(imported_instances.len(), 2);

    assert_eq!(imported_instances[0].0, meter_id);
    assert_eq!(imported_instances[0].1, "IMPORT-MULTI-001");
    assert_eq!(imported_instances[0].2, rust_decimal::Decimal::new(0, 3));

    assert_eq!(imported_instances[1].0, meter_id);
    assert_eq!(imported_instances[1].1, "IMPORT-MULTI-002");
    assert_eq!(
        imported_instances[1].2,
        rust_decimal::Decimal::new(100_000, 3)
    );

    cleanup_meter(&db, meter_id).await;
}
