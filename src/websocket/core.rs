use std::{collections::HashMap, env, net::SocketAddr, sync::{Arc, Mutex}};
use futures_util::StreamExt;
use tokio_tungstenite::{tungstenite::protocol::{frame::coding::CloseCode, CloseFrame}, WebSocketStream};
use crate::{model::User, types::WebSocketManager};
use http::{Request, Response};
use sqlx::{Pool, Postgres};
use tokio::net::{TcpListener, TcpStream};

pub async fn run_websocket_server(ws_manager: WebSocketManager, pool: Pool<Postgres>) {
  // Setting up listener
  let addr = env::var("WEBSOCKET_ADDRESS").unwrap_or(String::from("127.0.0.1:8040"));
  let listener = TcpListener::bind(
    &addr
  )
  .await
  .expect(format!("Can't listen to the address {}", addr).as_str());

  log::warn!("Web Socket listenning to wss://{}", addr);
  
  while let Ok((stream, addr)) = listener.accept().await {
    log::warn!("New WebSocket: {:?}", addr.ip());
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
  // Try to handle cookies and get the access token
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


  // Check if there's an error when accepting header of Web Socket initalization connection
  let mut ws_stream: WebSocketStream<TcpStream> = match ws_stream {
    Ok(ws) => ws,
    Err(err) => {
      log::error!("There's an error when trying to accept web socket connection to the device. Error: {}", err.to_string());
      return;
    }
  };


  // Verify access token
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


  // Get user data
  let user_data = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE access_token = $1",
    safe_access_token
  )
  .fetch_optional(&pool)
  .await;

  // Check if there's any error when trying to get user data from token
  let user_data = match user_data {
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

  // Check if there's user data
  match user_data {
    Some(_) => (),
    None => {
      log::warn!("Unauthorized access detected. Closed the connection from: {}", addr.ip());
      ws_stream.close(Some(CloseFrame {
        code: CloseCode::Policy,
        reason: "You're unauthorized".into()
      })).await.unwrap();
      return;
    }
  }


  let (ws_write, mut ws_read) = ws_stream.split();

  // Add connection to the list
  ws_manager.new_connection(safe_access_token.clone(), ws_write).await.unwrap();

  
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

          if text.starts_with("data") {
            println!("There's data : {}", text);
          }
          else if text.starts_with("test") {
            ws_manager.send_message(safe_access_token.clone(), String::from("TEST")).await.unwrap();
          }
          else {
            println!("There's random : {}", text);
          }
        }
      },
      Err(err) => {
        log::error!("There's an error found in a message. Error: {}", err.to_string());
      }
    }
  }
  
}