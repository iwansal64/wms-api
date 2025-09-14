use std::{collections::HashMap, sync::Arc};
use futures_util::{stream::SplitSink, SinkExt};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};


pub type WebSocketSender = Arc<RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>;


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

  pub async fn new_device_connection(&self, device_id: String, ws_sender: WebSocketSender) -> Result<(), String> {
    {
      let mut senders = self.device_senders.write().await;
      senders.insert(device_id.clone(), ws_sender);
    }
    log::info!("A new device connection has been added: {}", device_id);

    Ok(())
  }

  pub async fn new_user_connection(&self, device_id: String, addr: String, ws_sender: WebSocketSender) -> Result<(), String> {
    {
      let mut senders = self.user_senders.write().await;
      if let Some(senders_by_addr) = senders.get_mut(&device_id) {
        senders_by_addr.insert(addr, ws_sender);
      }
      else {
        let mut senders_by_addr: HashMap<String, WebSocketSender> = HashMap::new();
        senders_by_addr.insert(addr, ws_sender);
        senders.insert(device_id.clone(), senders_by_addr);
      }
    }
    log::info!("A new user connection has been added: {}", device_id);

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
    let mut sender_lock = sender.write().await;
    let send_result: Result<(), tokio_tungstenite::tungstenite::Error> = sender_lock.send(Message::Text(message.into())).await;

    
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

    log::debug!("CURRENT SENDERS: {:?}", senders_by_addr);
    // Iterate for each senders
    for sender in senders_by_addr.values() {
      // Send message
      let mut sender_lock = sender.write().await;
      let send_result: Result<(), tokio_tungstenite::tungstenite::Error> = sender_lock.send(Message::Text(message.into())).await;

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
    let senders_lock = self.user_senders.write().await;
    for senders_by_addr in senders_lock.values() {
      for sender in senders_by_addr.values() {
        let mut sender_lock = sender.write().await;
        sender_lock.close().await?;
      }
    }
    // Shutdown all device web sockets
    for device_sender in self.device_senders.write().await.values() {
      let mut device_sender_lock = device_sender.write().await;
      device_sender_lock.close().await?;
    }
    Ok(())
  }

  pub async fn remove_user_connection(&self, room_id: &str, addr: &str) -> Result<(), String> {
    // Lock the user senders write mode
    let mut user_senders_lock = self.user_senders.write().await;
    let user_senders_by_addr = user_senders_lock.get_mut(room_id);

    let user_senders_by_addr = match user_senders_by_addr {
      Some(data) => data,
      None => {
        return Err(format!("There's no user data with room_id: {}", room_id));
      }
    };

    let result = user_senders_by_addr.remove(addr);

    match result {
      Some(_) => Ok(()),
      None => Err(format!("There's no user data with addr: {} in this room_id: {}", addr, room_id))
    }
  }

  pub async fn remove_device_connection(&self, room_id: &str) -> Result<(), String> {
    // Lock the user senders write mode
    let mut device_senders_lock = self.device_senders.write().await;
    let result = device_senders_lock.remove(room_id);

    match result {
      Some(_) => Ok(()),
      None => Err(format!("There's no device data with room_id: {}", room_id))
    }
  }
}