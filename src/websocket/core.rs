use std::{collections::HashMap, env, net::SocketAddr, sync::{Arc, Mutex}};
use futures_util::{stream::SplitSink, StreamExt};
use tokio_tungstenite::{tungstenite::{self, protocol::{frame::coding::CloseCode, CloseFrame}}, WebSocketStream};
use crate::{model::{Connection, Device, User}, types::{WebSocketManager, WebSocketSender}};
use http::{Request, Response};
use sqlx::{Pool, Postgres};
use tokio::{net::{TcpListener, TcpStream}, sync::RwLock};
use either::Either;

pub async fn run_websocket_server(ws_manager: WebSocketManager, pool: Pool<Postgres>) {
  // Setting up listener
  let addr = env::var("WEBSOCKET_ADDRESS").unwrap_or(String::from("127.0.0.1:8040"));
  let listener = TcpListener::bind(
    &addr
  )
  .await
  .expect(format!("Can't listen to the address wss://{}", addr).as_str());

  log::warn!("Web Socket listenning to wss://{}", addr);
  
  while let Ok((stream, addr)) = listener.accept().await {
    log::warn!("({}:{}) WebSocket Client Connected", addr.ip(), addr.port());
    let ws_manager_instance: WebSocketManager = ws_manager.clone();
    let pool_instance: Pool<Postgres> = pool.clone();
    tokio::spawn(handle_websocket_connection(stream, ws_manager_instance, pool_instance, addr));
  }
}

fn handle_websocket_header_inspection(request: &Request<()>) -> Result<String, Response<Option<String>>> {
  // Get all of the cookies
  let mut cookies = HashMap::new();

  for (name, value) in request.headers() {
      if name.as_str().to_lowercase() != "cookie" {
        continue;
      }
      
      if let Ok(cookie_str) = value.to_str() {
        for cookie_pair in cookie_str.split(';') {
          let cookie_pair = cookie_pair.trim();
          if let Some((cookie_name, cookie_value)) = cookie_pair.split_once('=') {
            cookies.insert(
              cookie_name.trim().to_string(),
              cookie_value.trim().to_string()
            );
          }
        }
      }
  }

  // Get the access token
  let access_token = match cookies.get("access_token") {
    Some(token) => token,
    None => {
      return Err(Response::builder().status(401).body(Some(String::from("No Token Provided!"))).unwrap());
    }
  };


  Ok(access_token.clone())
}

async fn handle_websocket_connection(stream: TcpStream, ws_manager: WebSocketManager, pool: Pool<Postgres>, addr: SocketAddr) {
  //? Try to handle cookies and get the access token
  let access_token: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
  let access_token_instance = access_token.clone();
  let ws_stream = tokio_tungstenite::accept_hdr_async(
    stream, 
    move |request: &Request<()>, response: Response<()>| {
      
      // Get the cookie
      let result: Result<String, Response<Option<String>>> = handle_websocket_header_inspection(request);
      
      match result {
        Ok(token) => {
          let mut safe_access_token = access_token_instance.lock().unwrap();
          *safe_access_token = Some(token); // Exactly a token passed
          return Ok(response);
        },
        Err(err) => {
          return Err(err);
        }
      }
    }
  )
  .await;


  //? Check if there's an error when accepting header of Web Socket initalization connection
  let mut ws_stream: WebSocketStream<TcpStream> = match ws_stream {
    Ok(ws) => ws,
    Err(err) => {
      log::error!("There's an error when trying to accept web socket connection to the client. Error: {}", err.to_string());
      return;
    }
  };


  //? Get the access token
  let safe_access_token;
  {
    let raw_safe_access_token: Option<String> = match access_token.lock() {
      Ok(res) => Some(res.clone().unwrap()),
      Err(err) => {
        log::error!("There's an error when trying to get access token safely. Error: {}", err.to_string());
        None
      }
    };
  
    if raw_safe_access_token.is_none() {
      ws_stream.close(Some(CloseFrame {
        code: CloseCode::Error,
        reason: "There's an unexpected error".into()
      })).await.unwrap();
      return;
    }

    safe_access_token = String::from(raw_safe_access_token.unwrap());
  }


  //? Get user or device data
  let mut client_data: Option<either::Either<User, Device>> = None;
  let raw_user_data = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE access_token = $1",
    safe_access_token
  )
  .fetch_optional(&pool)
  .await;

  //? Check if there's any error when trying to get user data from token
  let user_data = match raw_user_data {
    Ok(data) => data,
    Err(err) => {
      log::error!("There's an error when trying to get user data. Error: {}", err.to_string());

      ws_stream.close(Some(CloseFrame {
        code: CloseCode::Error,
        reason: "There's an unexpected error".into()
      })).await.unwrap();
      return;
    }
  };

  //? Check if we get user data from the access token
  match user_data {
    Some(data) => {
      client_data = Some(either::Either::Left(data));
    },
    None => ()
  };

  if client_data.is_none() {
    let raw_device_data = sqlx::query_as!(
      Device,
      "SELECT * FROM devices WHERE access_token = $1",
      safe_access_token
    )
    .fetch_optional(&pool)
    .await;
  
    // Check if there's any error when trying to get user data from token
    let device_data = match raw_device_data {
      Ok(data) => data,
      Err(err) => {
        log::error!("There's an error when trying to get user data. Error: {}", err.to_string());
  
        ws_stream.close(Some(CloseFrame {
          code: CloseCode::Error,
          reason: "There's an unexpected error".into()
        })).await.unwrap();
        return;
      }
    };

    match device_data {
      Some(data) => {
        client_data = Some(either::Either::Right(data));
      },
      None => ()
    };
  }

  //? Check if there's device or user with that ID
  let client_data: either::Either<User, Device> = match client_data {
    Some(data) => data,
    None => {
      log::warn!("Unauthorized access detected. Closing the connection.");
      ws_stream.close(Some(CloseFrame {
        code: CloseCode::Policy,
        reason: "You're unauthorized".into()
      })).await.unwrap();
      return;
    }
  };
  

  //? Get the connection room id
  let connections_data: Result<Vec<Connection>, sqlx::Error>;
  let client_type: &str;
  match &client_data {
    either::Either::Left(user) => {
      // Get the connection ID query from user data
      client_type = "user";
      connections_data = sqlx::query_as!(
        Connection,
        "SELECT * FROM connections WHERE user_id = $1",
        user.id
      )
      .fetch_all(&pool)
      .await;
    },
    either::Either::Right(device) => {
      // Get the connection ID query from device data
      client_type = "device";
      connections_data = sqlx::query_as!(
        Connection,
        "SELECT * FROM connections WHERE device_id = $1",
        device.id
      )
      .fetch_all(&pool)
      .await;
    }
  }

  //? Check if there's any connection created just yet for the client?
  let connections_data: Vec<Connection> = match connections_data {
    Ok(res) => res,
    Err(err) => {
      log::error!("There's an error when trying to get connection data. Error: {}", err.to_string());
      ws_stream.close(Some(CloseFrame {
        code: CloseCode::Error,
        reason: "There's an unexpected error.".into()
      })).await.unwrap();
      return;
    }
  };

  //? Fetch the connection ID
  if connections_data.is_empty() {
    log::warn!("No connection formed yet");
    ws_stream.close(Some(CloseFrame {
      code: CloseCode::Policy,
      reason: "You're unauthorized".into()
    })).await.unwrap();
    return;
  }


  let (ws_write, mut ws_read) = ws_stream.split();
  let ws_client_address: String = format!("{}:{}", addr.ip(), addr.port());

  let ws_write: WebSocketSender = Arc::new(RwLock::new(SplitSink::from(ws_write).into()));

  // Add connection to the list
  for connection_data in &connections_data {
    match client_type {
      "user" => ws_manager.new_user_connection(connection_data.device_id.clone(), ws_client_address.clone(), ws_write.clone()).await.unwrap(),
      "device" => ws_manager.new_device_connection(connection_data.device_id.clone(), ws_write.clone()).await.unwrap(),
      _ => ()
    }
  }
 
  
  // Listen for incoming messages
  while let Some(raw_message) = ws_read.next().await {
    match raw_message {
      Ok(message) => {
        if message.is_text() {
          let text = match message.to_text() {
            Ok(res) => res,
            Err(_) => {
              continue;
            }
          };

          if client_type == "device" && text.starts_with("data") {
            let data = text.split(":").collect::<Vec<&str>>()[1];
            log::info!("Device is currently sending data: {}", data);
            let send_result: Result<(), String>;
            {
              let device_data = client_data.as_ref().expect_right("Makes no sense cause if the client type is 'device' the client data should be a device");
              send_result = ws_manager.send_user_message(&device_data.id, data).await;
            }

            match send_result {
              Ok(_) => {
                log::info!("Data has been successfully sent!");
              },
              Err(err) => {
                log::error!("There's an error when trying to send sensor data. Error: {}", err.to_string());
              }
            }
          } 
          else {
            log::info!("Get data from a {}: {}", client_type, text);
          }
        }
      },
      Err(err) => {
        match err {
          tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed | tungstenite::Error::Io(_) => {
            log::warn!("({}) Connection closed", ws_client_address);
          },
          _ => {
            log::error!("There's an error found in a message. Error: {}", err.to_string());
          }
        }
      }
    }
  }
  
  // After connection closed
  match client_data {
    Either::Left(_) => {
      for connection_data in connections_data {
        ws_manager.remove_user_connection(&connection_data.device_id, &ws_client_address).await.unwrap()
      }
    },
    Either::Right(device_data) => {
      ws_manager.remove_device_connection(&device_data.id).await.unwrap()
    }
  }
}