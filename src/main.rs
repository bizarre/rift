mod packet;
mod server;
mod command;
mod player;
mod engine;
mod config;
mod protocol;

use std::io;

use crate::server::ProxyServer;
use pretty_env_logger;
use log::{info};
use crate::engine::{Engine};
use crate::command::proxy::{ProxyCommand};
use crate::config::ProxyConfig;
use std::path::Path;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> io::Result<()> {
   std::env::set_var("RUST_LOG", "rift");

   pretty_env_logger::init_timed();

   info!("You're running rift v{} by Evercave.", VERSION);

   ProxyServer::new(move || {
       Engine::new()
        .command(ProxyCommand::default())
        .config(ProxyConfig::load(Path::new("./config.toml")))
   })
    .bind("localhost:25580")
    .await?
    .run()
    .await
}