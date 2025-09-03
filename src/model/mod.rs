use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct User {
  pub id: Uuid,
  pub username: String,
  pub password: String
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Connection {
  pub id: Uuid,
  pub topic: String,
}