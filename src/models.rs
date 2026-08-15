use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Meter {
    pub id: i64,
    pub name: String,
    pub unit: String,
}
