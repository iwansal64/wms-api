#[macro_use] extern crate rocket;

use std::env;
use gaia_api::{routes, db};
use dotenvy::dotenv;
use rocket::fairing::AdHoc;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    
    // Create the database pool
    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");
    
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