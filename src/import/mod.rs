use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
struct MeterCsvRow {
    name: String,
    unit: String,
}

#[derive(Debug, Deserialize)]
struct MeterInstanceCsvRow {
    meter_name: String,
    meter_number: String,
    initial_reading: Decimal,
    initial_reading_date: NaiveDate,
    installed_at: NaiveDate,
}

pub async fn import_meters(
    db: &PgPool,
    csv_data: &[u8],
) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_reader(csv_data);

    let mut transaction = db.begin().await?;
    let mut imported_meter_ids = Vec::new();

    for result in reader.deserialize() {
        let row: MeterCsvRow = result?;

        let meter_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO meters (name, unit)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(row.name)
        .bind(row.unit)
        .fetch_one(&mut *transaction)
        .await?;

        imported_meter_ids.push(meter_id);
    }

    transaction.commit().await?;

    Ok(imported_meter_ids)
}

pub async fn import_meter_instances(
    db: &PgPool,
    csv_data: &[u8],
) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_reader(csv_data);

    let mut transaction = db.begin().await?;
    let mut imported_meter_instance_ids = Vec::new();

    for result in reader.deserialize() {
        let row: MeterInstanceCsvRow = result?;

        // Find the logical meter referenced by its name.
        let meter_id: i64 = sqlx::query_scalar(
            r#"
            SELECT id
            FROM meters
            WHERE name = $1
            "#,
        )
        .bind(row.meter_name)
        .fetch_one(&mut *transaction)
        .await?;

        // Create the physical meter instance.
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
        .bind(row.meter_number)
        .bind(row.initial_reading)
        .bind(row.initial_reading_date)
        .bind(row.installed_at)
        .fetch_one(&mut *transaction)
        .await?;

        imported_meter_instance_ids.push(meter_instance_id);
    }

    transaction.commit().await?;

    Ok(imported_meter_instance_ids)
}
