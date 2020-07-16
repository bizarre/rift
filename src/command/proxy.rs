use crate::command::{Command, CommandSender};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const LABEL: &'static str = "proxy";
const ALIAS: &'static str = "rift";
pub struct ProxyCommand;

impl Command for ProxyCommand {
    fn get_label(&self) -> &'static str {
        LABEL
    }

    fn get_aliases(&self) -> Vec<&'static str> {
        vec![ALIAS]
    }

    fn is_console_only(&self) -> bool {
        true
    }

    fn execute(&self, sender: Box<dyn CommandSender>, mut arguments: Vec<String>) {
       if arguments.is_empty() {
           sender.send_message(format!("You are on proxy {}.", "Test"));
           return;
       }

       match arguments.pop() {
           Some(arg) => {
               match arg.to_lowercase().as_ref() {
                   "version" => {
                    sender.send_message(format!("Rift version {}", VERSION));
                   },

                   "stop" => {
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