use std::{collections::HashMap, sync::Arc};
use futures_util::{stream::SplitSink, SinkExt};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};


pub type WebSocketSender = SplitSink<WebSocketStream<TcpStream>, Message>;


#[derive(Clone)]
pub struct WebSocketManager {
  pub senders: Arc<RwLock<HashMap<String, WebSocketSender>>>
}


impl WebSocketManager {
  pub fn new() -> Self {
    Self {
      senders: Arc::new(RwLock::new(HashMap::new()))
    }
  }

  pub async fn new_connection(&self, id: String, ws_sender: WebSocketSender) -> Result<(), String> {
    {
      let mut senders = self.senders.write().await;
      senders.insert(id.clone(), ws_sender);
    }
    log::info!("A new connection has been added: {}", id);

    Ok(())
  }

  pub async fn send_message(&self, id: String, message: String) -> Result<(), String> {
    // Get the senders
    let raw_senders = Arc::clone(&self.senders);
    let mut write_mode_senders = raw_senders.write().await;
    

    // Get the sender from ID
    let sender: Option<&mut WebSocketSender> = write_mode_senders.get_mut(&id);


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

  pub async fn shutdown(&self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    for (_, sender) in self.senders.write().await.iter_mut() {
      sender.close().await?;
    }
    Ok(())
  }
}