use sqlx::PgPool;

#[derive(Debug, serde::Deserialize)]
struct MeterCsvRow {
    name: String,
    unit: String,
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
