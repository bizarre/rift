pub trait CommandSender {
    fn send_message<S: Into<String>>(&self, message: S);
    fn get_name(&self) -> &str;
}

