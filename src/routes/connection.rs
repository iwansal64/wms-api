use rocket::{get, http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConnectionResponseBody {
  
}


#[get("/connection")]
pub fn get() -> Result<Json<ConnectionResponseBody>, Status> {
    Err(Status::Locked)
}