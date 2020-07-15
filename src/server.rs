use std::{io, net};
use std::fmt::Display;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::BufReader;
use tokio::prelude::*;
use tokio::io::AsyncBufReadExt;
use std::io::BufRead;
use tokio::time::{Duration, Instant};
use log::{info, warn, debug};
use pretty_env_logger;
use crate::command::CommandSender;
use crate::player::Player;

pub struct ProxyServer {
    addresses: Vec<net::SocketAddr>,
    players: Vec<Player>,
    pub created_time: Instant
}

impl ProxyServer {
    pub fn new() -> Self {
        ProxyServer {
            addresses: Vec::new(),
            players: Vec::<Player>::new(),
            created_time: Instant::now()
        }
    }

    async fn attempt_bind<A: ToSocketAddrs>(&self, address: A) -> io::Result<Vec<TcpListener>> {
        let mut sockets = Vec::new();
        let mut successful = false;
        let mut error: Option<io::Error> = None;

        for address in address.to_socket_addrs().await? {
            match TcpListener::bind(address).await {
                Ok(socket) => { 
                    successful = true;
                    sockets.push(socket); 
                },
                Err(e) => error = Some(e)
            }
        }

        return if !successful {
            if let Some(e) = error.take() {
                Err(e)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to bind to address!",
                ))
            }
        } else {
            Ok(sockets)
        }
    }

    async fn listen(mut self, listener: TcpListener) -> io::Result<Self> {
        let address = listener.local_addr()?;
        self.addresses.push(address);

        debug!("Binded to address {}.", &address);

        Ok(self)
    }

    pub async fn bind<A: ToSocketAddrs + Display>(mut self, address: A) -> io::Result<Self> {
        let sockets = self.attempt_bind(address).await?;
        for listener in sockets {
            self = self.listen(listener).await?;
        }
        
        Ok(self)
    }

    pub fn run(self) -> ProxyServerRunner {
        if self.addresses.is_empty() {
            panic!("Must be bound to at least one address");
        }

        ProxyServerRunner {
            server: self
        }
    }

}

pub struct ProxyServerRunner {
    server: ProxyServer
}

impl Future for ProxyServerRunner {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        info!("Successfully started in {:?}.", this.server.created_time.elapsed());

        tokio::spawn(async move {
            let stdin = io::stdin();
            let mut stdin = io::BufReader::new(stdin);
            loop {
                let mut line = String::new();
                stdin.read_line(&mut line).unwrap();
                print!("{}", line);
            }
        });

        Poll::Pending
    }
}

impl CommandSender for ProxyServer {
    fn get_name(&self) -> &str {
        "Console"
    }

    fn send_message<S: Into<String>>(&self, message: S) {
        println!("{}", message.into())
    }
}