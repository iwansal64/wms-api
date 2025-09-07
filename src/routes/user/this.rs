use rocket::{get, http::{CookieJar, Status}, serde::json::Json, State, time::PrimitiveDateTime};
use serde::{Deserialize, Serialize};
use crate::model::custom_serde;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExposedUser {
  pub id: String,
  pub username: Option<String>,
  pub email: String,
  #[serde(with = "custom_serde::primitive_datetime")]
  pub created_at: PrimitiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserRequestBody {
  pub user_data: ExposedUser
}

#[get("/user/get")]
pub async fn get(cookies: &CookieJar<'_>, db: &State<sqlx::postgres::PgPool>) -> Result<Json<GetUserRequestBody>, Status> {
  // Get the access token
  let access_token: Option<&rocket::http::Cookie<'static>> = cookies.get("access_token");

  let access_token: String = match access_token {
    Some(token) => token.value().to_string(),
    None => {
      return Err(Status::Unauthorized);
    }
  };
  
  // Get the user data
  let user_data: ExposedUser;
  {
    let raw_user_data: Result<Option<ExposedUser>, sqlx::Error> = sqlx::query_as!(
      ExposedUser,
      "SELECT id, username, email, created_at FROM users WHERE access_token = $1",
      access_token
    )
    .fetch_optional(db.inner())
    .await;
  
    user_data = match raw_user_data {
      Ok(res) => match res {
        Some(data) => data,
        None => {
          return Err(Status::Unauthorized);
        }
      },
      Err(err) => {
        log::error!("There's an error when trying to get user data. Error: {}", err.to_string());
        return Err(Status::InternalServerError);
      }
    }
  }


  // Return user data as the response
  Ok(Json(GetUserRequestBody { user_data: user_data }))
}