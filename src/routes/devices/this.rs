use rocket::{get, http::{CookieJar, Status}, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::model::{Device, User, Connection};

#[derive(Serialize, Deserialize)]
pub struct GetReturnType {
  devices: Vec<Device>
}

#[get("/device")]
pub async fn get(cookies: &CookieJar<'_>, db: &State<Pool<Postgres>>) -> Result<Json<GetReturnType>, Status> {
  // Get the access token
  let access_token = cookies.get("access_token");

  let access_token = match access_token {
    Some(token) => token.value(),
    None => {
      return Err(Status::Unauthorized);
    }
  };


  // Verify access token
  let user_data: User;
  {
    let raw_user_data: Result<Option<User>, sqlx::Error> = sqlx::query_as!(
      User,
      "SELECT * FROM users WHERE access_token = $1",
      access_token
    )
    .fetch_optional(db.inner())
    .await;

    let optional_user_data: Option<User> = match raw_user_data {
      Ok(res) => res,
      Err(err) => {
        log::error!("There's an error when trying to get user data for access verification. Error: {}", err.to_string());
        return Err(Status::InternalServerError);
      }
    };

    user_data = match optional_user_data {
      Some(user) => user,
      None => {
        return Err(Status::Unauthorized);
      }
    }
  }
  


  // Get the device data
  let mut devices_data: Vec<Device> = Vec::new();
  {
    // Get all connections that user connected
    let raw_connections_data: Result<Vec<Connection>, sqlx::Error> = sqlx::query_as!(
      Connection,
      "SELECT * FROM connections WHERE user_id = $1",
      user_data.id
    )
    .fetch_all(db.inner())
    .await;

    let connections_data = match raw_connections_data {
      Ok(data) => data,
      Err(err) => {
        log::error!("There's an error when trying to get connections data. Error: {}", err.to_string());
        return Err(Status::InternalServerError);
      }
    };

    // Get all the devices from each connection
    for connection in connections_data {
      let raw_device_data: Result<Option<Device>, sqlx::Error> = sqlx::query_as!(
        Device,
        "SELECT * FROM devices WHERE id = $1",
        connection.device_id
      )
      .fetch_optional(db.inner())
      .await;

      let optional_device_data: Option<Device> = match raw_device_data {
        Ok(device) => device,
        Err(err) => {
          log::error!("There's an error when trying to get device data for each connection. Error: {}", err.to_string());
          return Err(Status::InternalServerError);
        }
      };

      let device_data: Device = match optional_device_data {
        Some(device) => device,
        None => {
          log::error!("LOGIC ERROR: There's changes in database in the middle of program. When trying to get all connections. there's device that has their ID change or deleted.");
          return Err(Status::ServiceUnavailable);
        }
      };

      devices_data.push(device_data);
    }
  }


  // Return the device data
  Ok(Json(GetReturnType {
    devices: devices_data
  }))
}