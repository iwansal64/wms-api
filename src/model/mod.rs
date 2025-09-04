use serde::{Serialize, Deserialize};
use chrono::{DateTime, TimeZone, Utc};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DateTimeSQLX {
  inner: DateTime<Utc>
}

impl From<DateTime<Utc>> for DateTimeSQLX {
  fn from(dt: DateTime<Utc>) -> Self {
    DateTimeSQLX { inner: dt } 
  }
}

impl From<sqlx::types::time::PrimitiveDateTime> for DateTimeSQLX {
  fn from(value: sqlx::types::time::PrimitiveDateTime) -> Self {
    DateTimeSQLX { inner: Utc.timestamp_opt(value.assume_utc().unix_timestamp(), 0).unwrap() }
  }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
  pub id: String,
  pub username: String,
  pub password: String,
  pub email: String,
  pub created_at: DateTimeSQLX
}