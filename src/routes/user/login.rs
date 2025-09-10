use rocket::{
    http::{Cookie, CookieJar, Status}, post, serde::{json::Json, Deserialize}, State
};
use sqlx::{Pool, Postgres};

use crate::{model::User, util::generate_token};
use sha3::{Digest, Sha3_256};
use hex;
use log;


#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

// FUNCTIONS
#[post("/user/login", data = "<credentials>")]
pub async fn post(credentials: Json<LoginRequest>, db: &State<Pool<Postgres>>, cookies: &CookieJar<'_>) -> Result<(), Status> {
    // Get the user data from database
    let sqlx_query_result = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        credentials.username
    ).fetch_optional(db.inner())
    .await;


    let user_data = match sqlx_query_result {
        Ok(user_data) => user_data,
        Err(err) => {
            log::error!("There's an error when trying to get user data, error: {}", err.to_string());
            return Err(Status::InternalServerError);
        }  
    };

    
    // Verify the user data
    let user_data = match user_data {
        Some(data) => data,
        None => {
            return Err(Status::NotFound);
        }
    };

    if let None = user_data.password {
        return Err(Status::NotFound);
    }

    
    // Get the hash version of the given password
    let hash_result;
    {
        let mut hasher = Sha3_256::new();
        hasher.update(credentials.password.as_bytes());
        
        let hash_raw_result = hasher.finalize();
        
        hash_result = hex::encode(hash_raw_result);
    }

    
    // Verify the password
    if hash_result != user_data.password.unwrap() {
        log::warn!("There's a failed attempt to login for user: {}", credentials.username);
        return Err(Status::Unauthorized);
    }


    // Generate token
    let generated_token: String = generate_token();


    // Update access token in database
    let token_raw_result = sqlx::query!(
        "UPDATE users SET access_token = $1 WHERE id = $2",
        generated_token,
        user_data.id
    )
    .execute(db.inner())
    .await;

    match token_raw_result {
        Ok(_) => (),
        Err(err) => {
            log::error!("There's an error when trying to insert token data. Error: {}", err.to_string());
            return Err(Status::InternalServerError);
        }
    }


    // Store access token to the cookie
    cookies.add(
        Cookie::build(("access_token", generated_token))
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(rocket::time::Duration::days(14))
    );
    

    // Return the token
    Ok(())
}