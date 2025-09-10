use rocket::{http::{Cookie, CookieJar, Status}, post, serde::json::Json, time::{macros::datetime, OffsetDateTime, PrimitiveDateTime}, State};
use serde::{Serialize, Deserialize};
use sha3::{Sha3_256, Digest};

use crate::util::{generate_token, is_duplicated_error};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegistrationRequestType {
  verification_token: String,
  username: String,
  password: String
}


#[post("/user/register/2", data = "<registration_data>")]
pub async fn post(registration_data: Json<RegistrationRequestType>, cookies: &CookieJar<'_>, db: &State<sqlx::postgres::PgPool>) -> Result<(), Status> {
  // Verify the registration token
  {
    let selected_user_data = sqlx::query!(
      "SELECT * FROM users WHERE verification_token = $1",
      registration_data.verification_token
    )
    .fetch_optional(db.inner())
    .await;

    match selected_user_data {
      Ok(res) => match res {
        Some(data) => data,
        None => {
          return Err(Status::Unauthorized);
        }
      },
      Err(err) => {
        log::error!("There's an error when trying to get the user data. Error: {}", err.to_string());
        return Err(Status::InternalServerError);
      }
    };
  }

  
  // Hash the given password
  let hashed_password: String;
  {
    let mut hasher = Sha3_256::new();
    hasher.update(registration_data.password.as_bytes());
    
    let hash_raw_result = hasher.finalize();
    
    hashed_password = hex::encode(hash_raw_result);
  }


  // Generate access token
  let generated_access_token: String = generate_token();
  let current_date: OffsetDateTime = OffsetDateTime::now_utc();
  let access_token_expire_date: PrimitiveDateTime = datetime!(2000-01-01 00:00:00).replace_date(current_date.date().saturating_add(time::Duration::days(14)));
  
  // Update the data in database
  {
    let update_result = sqlx::query!(
      "UPDATE users SET username = $1, password = $2, access_token = $3, access_token_expire = $4, verification_token = NULL WHERE verification_token = $5",
      registration_data.username,
      hashed_password,
      generated_access_token,
      access_token_expire_date,
      registration_data.verification_token
    )
    .execute(db.inner())
    .await;
  
    match update_result {
      Ok(res) => res,
      Err(err) => {
        if is_duplicated_error(&err) {
          return Err(Status::Conflict);
        }
        
        log::error!("There's an error when trying to update the user data in registration. Error: {}", err.to_string());
        return Err(Status::InternalServerError);
      }
    };
  }


  // Create cookie
  cookies.add(
    Cookie::build(("access_token", generated_access_token))
    .path("/")
    .secure(true)
    .http_only(true)
    .max_age(rocket::time::Duration::days(14))
  );
  

  // Return OK Response
  Ok(())
}