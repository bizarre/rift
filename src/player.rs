use crate::command::CommandSender;
use uuid::Uuid;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Player {
    id: Uuid,
    name: String
}

impl Player {
    pub fn new<S: Into<String>>(id: Uuid, name: S) -> Self {
        Player {
            id: id,
            name: name.into()
        }
    }
}

impl CommandSender for Player {
    fn get_name(&self) -> &str {
        &self.name
    }
 
    fn send_message(&self, message: String) {
        // todo: send msg
    }
}