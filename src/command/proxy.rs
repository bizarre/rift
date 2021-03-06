use crate::command::{Command, CommandSender};
use std::io;
use crate::server::Server;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const LABEL: &'static str = "proxy";
const ALIAS: &'static str = "rift";
pub struct ProxyCommand {
    backend: Option<Box<dyn Server + Send + Sync>>
}

impl std::default::Default for ProxyCommand {
    fn default() -> Self {
        ProxyCommand {
            backend: None
        }
    }
}

impl Command for ProxyCommand {
    fn get_label(&self) -> &'static str {
        LABEL
    }

    fn get_aliases(&self) -> Vec<&'static str> {
        vec![ALIAS]
    }

    fn is_console_only(&self) -> bool {
        false
    }

    fn set_backend(&mut self, server: Box<dyn Server + Send + Sync>) -> io::Result<()> {
        self.backend = Some(server);
        Ok(())
    }

    fn execute(&self, sender: Box<dyn CommandSender>, mut arguments: Vec<String>) {
       if arguments.is_empty() {
           sender.send_message(format!("You are on proxy {}.", "Test"));
           return;
       }

       match arguments.pop() {
           Some(arg) => {
               match arg.to_lowercase().as_ref() {
                   "version" | "ver" => {
                    sender.send_message(format!("Rift version {}", VERSION));
                   },

                   "stop" | "end" | "kill" | "shutdown" => {
                    sender.send_message(String::from("Stopping the proxy server.."));
                   }

                   "list" => {
                       sender.send_message(String::from("Players: "))
                   }
                   
                   _ => {
                    sender.send_message(String::from("Unknown proxy command."));
                   }
               }
           },
           _ => {}
       }
    }
}