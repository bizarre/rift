use crate::command::{Command, CommandSender};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const LABEL: &'static str = "version";
const ALIAS: &'static str = "ver";
pub struct VersionCommand;

impl Command for VersionCommand {
    fn get_label(&self) -> &'static str {
        LABEL
    }

    fn get_aliases(&self) -> Vec<&'static str> {
        vec![ALIAS]
    }

    fn is_console_only(&self) -> bool {
        true
    }

    fn execute(&self, sender: Box<dyn CommandSender>, arguments: Vec<String>) {
        sender.send_message(format!("version: {}", VERSION));
    }
}