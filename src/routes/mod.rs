use rocket::{
    get,
    http::Status,
    post,
    serde::{Deserialize, Serialize, json::Json},
    State,
};
use sqlx::{Pool, Postgres};

// DATA SCHEMA
#[derive(Serialize, Deserialize)]
pub struct GetUserResponseType {
    topic: String,
}
#[derive(Serialize, Deserialize)]
pub struct ConnectionResponseType {
    topic: String,
}
#[derive(Serialize, Deserialize)]
pub struct LoginResponseType {
    topic: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

// FUNCTIONS
#[post("/user/login", data = "<credentials>")]
pub async fn login(credentials: Json<LoginRequest>, db: &State<Pool<Postgres>>) -> Result<Json<ConnectionResponseType>, Status> {
    // Example of using SQLx to query the database
    Ok(Json(ConnectionResponseType {
      topic: "".to_string()
    }))
}

#[get("/user/get")]
pub fn get_user() -> Result<Json<GetUserResponseType>, Status> {
    Err(Status::Locked)
}

#[get("/connection")]
pub fn get_connection() -> Result<Json<ConnectionResponseType>, Status> {
    Err(Status::Locked)
}
