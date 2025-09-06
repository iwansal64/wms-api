use rocket::{get, http::Status, serde::json::Json};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ConnectionResponseType {
  
}


#[get("/user/register/1")]
pub fn get() -> Result<Json<ConnectionResponseType>, Status> {
  Err(Status::Locked)
}