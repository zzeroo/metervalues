use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize)]
pub struct RemoveMeterInstance {
    pub removed_at: NaiveDate,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Meter {
    pub id: i64,
    pub name: String,
    pub unit: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMeterInstance {
    pub meter_number: String,
    pub initial_reading: Decimal,
    pub initial_reading_date: NaiveDate,
    pub installed_at: NaiveDate,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MeterInstance {
    pub id: i64,
    pub meter_id: i64,
    pub meter_number: String,
    pub initial_reading: Decimal,
    pub initial_reading_date: NaiveDate,
    pub installed_at: NaiveDate,
    pub removed_at: Option<NaiveDate>,
}
