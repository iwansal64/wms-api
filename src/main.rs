#[macro_use] extern crate rocket;

use std::env;
use gaia_api::routes;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        // customize logging format
        .format(|out, message, record| {
            out.finish(format_args!("({}) [{}] {}", record.level(), chrono::Local::now().format("%H:%M:%S"), message));
        })
        // set a global minimum logging level
        .level(log::LevelFilter::Warn)
        // add handler for stdout
        .chain(std::io::stdout())
        // add handler for a log file
        .chain(fern::log_file(format!("logs/{}.log", chrono::Local::now().format("%d-%m-%Y")))?)
        // apply the configuration
        .apply()?;

    Ok(())
}


#[launch]
async fn rocket() -> _ {
    // Initializions
    dotenv().ok();
    setup_logger().expect("There's an error when trying to setup logger");
    
    // Connect to PostgreSQL
    let pool: Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            env::var("DATABASE_URL")
            .expect("Database URL isn't specified yet.")
            .as_str()
        )
        .await;

    let pool: sqlx::Pool<sqlx::Postgres> = match pool {
        Ok(res) => res,
        Err(err) => {
            log::error!("Error when setting up database connection. Error: {}", err.to_string());
            panic!("There's an error when setting up database connection.");
        }
    };

    
    rocket::build()
        // Setting up postgresql pool for database connection
        .manage(pool)
        // Konfigurasi rocket
        .configure(
            rocket::Config::figment()
            // Setting up port
            .merge((
                "port",
                env::var("PORT")
                .unwrap_or(String::from("8080"))
                .parse::<u16>()
                .expect("PORT must be a valid number"),
            ))
            // Setting up cli color to false
            .merge((
                "cli_colors",
                env::var("CLI_COLORS")
                .unwrap_or(String::from("false"))
            ))
        )
        // Put all of the routes
        .mount("/", routes![
            routes::connection::get,
            routes::user::this::get,
            routes::user::login::post,
            routes::user::verify_email::post,
            routes::user::registration::post
        ])
        // Register catchers
        .register("/", catchers![
            routes::catchers::not_found,
            routes::catchers::unauthorized,
            routes::catchers::too_many_requests
        ])
}