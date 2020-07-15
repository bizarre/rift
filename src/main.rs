mod packet;
mod server;
mod command;
mod player;
mod engine;
mod config;

use std::io;

use crate::server::ProxyServer;
use pretty_env_logger;
use log::{info};
use crate::engine::{Engine};
use crate::command::version::{VersionCommand};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> io::Result<()> {
   std::env::set_var("RUST_LOG", "rift");

   pretty_env_logger::init_timed();

   info!("version - {}", VERSION);

   ProxyServer::new(move || {
       Engine::new()
        .command(VersionCommand)
   })
    .bind("localhost:25565")
    .await?
    .run()
    .await
}