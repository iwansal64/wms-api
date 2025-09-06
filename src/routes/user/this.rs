use rocket::{get, http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetUserRequestBody {
  
}

#[get("/user/get")]
pub fn get() -> Result<Json<GetUserRequestBody>, Status> {
  Err(Status::Locked)
}