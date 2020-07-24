mod packet;
mod server;
mod command;
mod player;
mod engine;
mod config;
mod protocol;
mod util;

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

   info!("You're running rift v{}.", VERSION);

   let mut config = ProxyConfig::load(Path::new("./config.toml"));
   let bind = config.bind;

   let path = Path::new("./favicon.png");
   if path.exists() {
        config.set_favicon(image_base64::to_base64("./favicon.png"));
   }

   ProxyServer::new(move || {
        let cloned = config.clone();
        
        Engine::new()
          .command(ProxyCommand::default())
          .config(cloned)
   })
    .bind(bind)
    .await?
    .run()
    .await
}