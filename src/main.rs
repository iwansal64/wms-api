#[macro_use] extern crate rocket;

use std::{env, net::IpAddr};
use wms_api::{routes, types::WebSocketManager, websocket};
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::spawn;

 
fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        // customize logging format
        .format(|out, message, record| {
            out.finish(format_args!("({}) [{}] {}", record.level(), chrono::Local::now().format("%H:%M:%S"), message));
        })
        // set a global minimum logging level
        .level(log::LevelFilter::Info)
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

    let pool: Pool<Postgres> = match pool {
        Ok(res) => res,
        Err(err) => {
            log::error!("Error when setting up database connection. Error: {}", err.to_string());
            panic!("There's an error when setting up database connection.");
        }
    };

    let ws_manager: WebSocketManager = WebSocketManager::new();

    let ws_manager_instance: WebSocketManager = ws_manager.clone();
    let pool_instance: Pool<Postgres> = pool.clone();
    let ws_manager_instance_2: WebSocketManager = ws_manager.clone();
    spawn(async move {
        tokio::select! {
            _ = websocket::core::run_websocket_server(ws_manager_instance, pool_instance) => (),
            _ = tokio::signal::ctrl_c() => {
                let shutdown_result = ws_manager_instance_2.shutdown().await;
                match shutdown_result {
                    Ok(_) => (),
                    Err(err) => {
                        log::error!("There's an error when shutting down all of the websocket connection. Error: {}", err.to_string());
                    }
                }
            }
        }
    });
    
    rocket::build()
        // Setting up postgresql pool for database connection
        .manage(pool)
        // Setting up web socket manager for web socket connection
        .manage(ws_manager)
        // Konfigurasi rocket
        .configure(
            rocket::Config::figment()
            // Setting up address
            .merge((
                "address",
                env::var("API_ADDRESS")
                .unwrap_or(String::from("127.0.0.1"))
                .parse::<IpAddr>()
                .expect("API_ADDRESS must be a valid IP address"),
            ))
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
            routes::user::this::get,
            routes::user::login::post,
            routes::user::register_email::post,
            routes::user::verify_email::post,
            routes::user::create_user::post,
            routes::devices::this::get
        ])
        // Register catchers
        .register("/", catchers![
            routes::catchers::not_found,
            routes::catchers::unauthorized,
            routes::catchers::too_many_requests
        ])
}