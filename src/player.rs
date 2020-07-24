use crate::command::CommandSender;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<HashMap<String, String>>,
    #[serde(skip_serializing)]
    pub server: Option<String>
}

impl Player {
    pub fn new<S: Into<String>>(id: Uuid, name: S) -> Self {
        Player {
            id: id,
            name: name.into(),
            properties: Vec::new(),
            server: None
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