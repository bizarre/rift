mod packet;
mod server;
use std::io;

use crate::server::ProxyServer;

#[tokio::main]
async fn main() -> io::Result<()> {
   ProxyServer::new()
    .bind("localhost:25565")
    .await?
    .run()
    .await
}