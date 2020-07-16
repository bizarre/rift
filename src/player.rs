use crate::command::CommandSender;
use uuid::Uuid;
use tokio::net::TcpStream;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Player {
    uuid: Uuid,
    name: String
}

impl Player {

}

impl CommandSender for Player {
    fn get_name(&self) -> &str {
        &self.name
    }
 
    fn send_message(&self, message: String) {
        // todo: send msg
    }
}