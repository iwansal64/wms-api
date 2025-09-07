use rocket::time::PrimitiveDateTime;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
pub mod custom_serde;

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct User {
  pub id: String,
  pub username: Option<String>,
  pub password: Option<String>,
  pub email: String,
  pub verification_token: Option<String>,
  pub access_token: Option<String>,

  #[serde(with = "custom_serde::primitive_datetime")]
  pub created_at: PrimitiveDateTime,
  #[serde(with = "custom_serde::optional_primitive_datetime")]
  pub access_token_expire: Option<PrimitiveDateTime>
}