use rocket::{http::{CookieJar, Status}, post, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use sha3::{Sha3_256, Digest};
use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use crate::model::User;


#[derive(Serialize, Deserialize)]
pub struct CreateUserRequestType {
  username: String,
  password: String
}


#[post("/user/register/3", data = "<create_user_data>")]
pub async fn post(create_user_data: Json<CreateUserRequestType>, db: &State<Pool<Postgres>>, cookies: &CookieJar<'_>) -> Result<(), Status> {
  // Check if access token exists
  let access_token = cookies.get("access_token");
  let access_token = match access_token {
    Some(cookie) => cookie.value(),
    None => {
      log::warn!("Unauthorized out of no access token in their cookie!");
      return Err(Status::Unauthorized);
    }
  };


  // Verify access token
  {
    // Verify if the access token is inside the database
    let raw_user_data: Result<Option<User>, sqlx::Error> = sqlx::query_as!(
      User,
      "SELECT * FROM users WHERE access_token = $1",
      access_token
    )
    .fetch_optional(db.inner())
    .await;

    let user_data: Option<User> = match raw_user_data {
      Ok(data) => data,
      Err(err) => {
        log::error!("There's an error when trying to verify access token while creating user process. Error: {}", err.to_string());
        return Err(Status::InternalServerError);
      } 
    };

    let user_data = match user_data {
      Some(data) => data,
      None => {
        log::warn!("Unauthorized out of not valid access token! Access Token: {}", access_token);
        return Err(Status::Unauthorized);
      }
    };

    // Verify if the user is not already registered before
    if user_data.username.is_some() && user_data.password.is_some() {
      log::warn!("Unauthorized out of duplicated registering!");
      return Err(Status::Unauthorized);
    }
  }

  
  // Hash the given password
  let hashed_password: String;
  {
    let mut hasher = Sha3_256::new();
    hasher.update(create_user_data.password.as_bytes());
    
    let hash_raw_result = hasher.finalize();
    
    hashed_password = hex::encode(hash_raw_result);
  }

  // Update the username and password
  {
    let raw_user_data: Result<PgQueryResult, sqlx::Error> = sqlx::query!(
      "UPDATE users SET username = $1, password = $2 WHERE access_token = $3",
      create_user_data.username,
      hashed_password,
      access_token
    )
    .execute(db.inner())
    .await;

    match raw_user_data {
      Ok(_) => (),
      Err(err) => {
        log::error!("There's an error when trying to update user while creating user process. Error: {}", err.to_string());
        return Err(Status::InternalServerError);
      }
    }
  }

  // Return OK Response
  Ok(())
}


