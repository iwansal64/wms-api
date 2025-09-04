#[macro_use] extern crate rocket;

use std::env;
use gaia_api::routes;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    
    // Connect to PostgreSQL
    let pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            env::var("DATABASE_URL")
            .expect("Database URL isn't specified yet.")
            .as_str()
        )
        .await
        .expect("Connection failed");
                
        
    rocket::build()
        .manage(pool)
        .configure(rocket::Config::figment().merge((
            "port",
            env::var("PORT")
            .expect("Check your ENV bruh")
            .parse::<u16>()
            .expect("PORT must be a valid number"),
        )))
        .mount("/", routes![
            routes::get_connection,
            routes::get_user,
            routes::login
        ])
}