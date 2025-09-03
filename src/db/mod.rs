use sqlx::{Pool, Postgres, query_as};
use std::env;

pub async fn create_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    Pool::connect(&database_url).await
}

// Example of a query function
pub async fn find_user_by_username(pool: &Pool<Postgres>, username: &str) -> Result<Option<User>, sqlx::Error> {
    query_as!(
        User,
        "SELECT id, username, email FROM users WHERE username = $1",
        username
    )
    .fetch_optional(pool)
    .await
}

// Example of a User struct for the database
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
}