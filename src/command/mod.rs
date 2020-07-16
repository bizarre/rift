pub mod proxy;
use std::io;
use crate::server::Server;

pub trait CommandSender {
    fn send_message(&self, message: String);
    fn get_name(&self) -> &str;
}

pub trait CommandExecutor {
    fn parse(sender: Box<dyn CommandSender>, command: String);
}

pub trait Command {
    fn get_label(&self) -> &'static str;
    fn get_aliases(&self) -> Vec<&'static str>;
    fn is_console_only(&self) -> bool;
    fn execute(&self, sender: Box<dyn CommandSender>, arguments: Vec<String>);
    fn set_backend(&mut self, server: Box<dyn Server +  Send + Sync>) -> io::Result<()>;
}

pub struct ProxyCommandExecutor {

}

impl CommandExecutor for ProxyCommandExecutor {
    fn parse(sender: Box<dyn CommandSender>, command: String) {

    }
}