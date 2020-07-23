use crate::command::CommandSender;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct Player {
    id: Uuid,
    name: String,
    properties: Vec<HashMap<String, String>>
}

impl Player {
    pub fn new<S: Into<String>>(id: Uuid, name: S) -> Self {
        Player {
            id: id,
            name: name.into(),
            properties: Vec::new()
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