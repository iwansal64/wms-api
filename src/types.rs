use std::{collections::HashMap, sync::Arc};
use futures_util::{stream::SplitSink, SinkExt};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};


pub type WebSocketSender = SplitSink<WebSocketStream<TcpStream>, Message>;


#[derive(Clone)]
pub struct WebSocketManager {
  pub user_senders: Arc<RwLock<HashMap<String, HashMap<String, WebSocketSender>>>>,
  pub device_senders: Arc<RwLock<HashMap<String, WebSocketSender>>>
}


impl WebSocketManager {
  pub fn new() -> Self {
    Self {
      user_senders: Arc::new(RwLock::new(HashMap::new())),
      device_senders: Arc::new(RwLock::new(HashMap::new()))
    }
  }

  pub async fn new_device_connection(&self, room_id: String, ws_sender: WebSocketSender) -> Result<(), String> {
    {
      let mut senders = self.device_senders.write().await;
      senders.insert(room_id.clone(), ws_sender);
    }
    log::warn!("A new device connection has been added: {}", room_id);

    Ok(())
  }

  pub async fn new_user_connection(&self, room_id: String, addr: String, ws_sender: WebSocketSender) -> Result<(), String> {
    {
      let mut senders = self.user_senders.write().await;
      if let Some(senders_by_addr) = senders.get_mut(&room_id) {
        senders_by_addr.insert(addr, ws_sender);
      }
      else {
        let mut senders_by_addr: HashMap<String, WebSocketSender> = HashMap::new();
        senders_by_addr.insert(addr, ws_sender);
        senders.insert(room_id.clone(), senders_by_addr);
      }
    }
    log::warn!("A new user connection has been added: {}", room_id);

    Ok(())
  }

  pub async fn send_device_message(&self, id: &str, message: &str) -> Result<(), String> {
    // Get the senders
    let raw_senders = Arc::clone(&self.device_senders);
    let mut write_mode_senders = raw_senders.write().await;
    

    // Get the sender from ID
    let sender: Option<&mut WebSocketSender> = write_mode_senders.get_mut(id);


    // Check if the sender for that ID is exists
    let sender: &mut WebSocketSender = match sender {
      Some(data) => data,
      None => {
        return Err(format!("There's no recorded web socket connection with ID: {}", id));
      }
    };

    // Send message
    let send_result: Result<(), tokio_tungstenite::tungstenite::Error> = sender.send(Message::Text(message.into())).await;

    
    // Check if there's an error
    match send_result {
      Ok(_) => (),
      Err(err) => {
        let err_message = format!("There's an error when trying to send data through Web Socket. Error: {}", err.to_string());
        log::error!("{}", err_message);
        return Err(err_message);
      }
    }
    
    Ok(())
  }

  pub async fn send_user_message(&self, id: &str, message: &str) -> Result<(), String> {
    // Get the senders
    let raw_user_senders = Arc::clone(&self.user_senders);
    let mut write_mode_user_senders = raw_user_senders.write().await;

    // Get all of the user senders from ID
    let raw_senders: Option<&mut HashMap<String, WebSocketSender>> = write_mode_user_senders.get_mut(id);


    // Check if the values from that ID is exists
    let senders_by_addr: &mut HashMap<String, WebSocketSender> = match raw_senders {
      Some(data) => data,
      None => {
        return Err(format!("There's no recorded web socket connection with ID: {}", id));
      }
    };

    log::warn!("CURRENT SENDERS: {:?}", senders_by_addr);
    // Iterate for each senders
    for sender in senders_by_addr.values_mut() {
      // Send message
      let send_result: Result<(), tokio_tungstenite::tungstenite::Error> = sender.send(Message::Text(message.into())).await;

      // Check if there's an error
      match send_result {
        Ok(_) => (),
        Err(err) => {
          let err_message = format!("There's an error when trying to send data through Web Socket. Error: {}", err.to_string());
          log::error!("{}", err_message);
          return Err(err_message);
        }
      }
    }
    
    
    Ok(())
  }

  pub async fn shutdown(&self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    // Shutdown all user web sockets
    let mut senders_lock = self.user_senders.write().await;
    for senders_by_addr in senders_lock.values_mut() {
      for sender in senders_by_addr.values_mut() {
        sender.close().await?;
      }
    }
    // Shutdown all device web sockets
    for device_sender in self.device_senders.write().await.values_mut() {
      device_sender.close().await?;
    }
    Ok(())
  }
}