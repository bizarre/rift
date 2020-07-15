mod packet;
mod server;
mod command;
mod player;

use std::io;

use crate::server::ProxyServer;
use pretty_env_logger;
use log::{info};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> io::Result<()> {
   std::env::set_var("RUST_LOG", "debug");
   pretty_env_logger::init();

   info!("version - {}", VERSION);

   ProxyServer::new()
    .bind("localhost:25565")
    .await?
    .run()
    .await
}