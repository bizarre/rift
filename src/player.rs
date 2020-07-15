use crate::command::CommandSender;
use uuid::Uuid;

pub struct Player {
    uuid: Uuid,
    name: String
}

impl CommandSender for Player {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn send_message(&self, message: String) {
        // todo: send msg
    }
}