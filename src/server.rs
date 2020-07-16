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
use log::{info, warn, debug, error, trace};
use pretty_env_logger;
use crate::command::{CommandSender, CommandExecutor, ProxyCommandExecutor};
use crate::player::Player;
use crate::engine::{ProxyEngine, IntoProxyEngine};
use crate::config::{ProxyConfig};
use std::marker::PhantomData;
use uuid::Uuid;
use crate::packet::{Packet, In, AsyncPacketReadExt, AsyncPacketWriteExt, Out};
use crate::packet::handshake;

pub trait Server {
    fn get_players(&self) -> Vec<Player>;
    fn get_addresses(&self) -> Vec<net::SocketAddr>;
}

#[derive(Clone)]
struct DynServer {
    players: Vec<Player>,
    addresses: Vec<net::SocketAddr>
}

pub struct ProxyServer<F, I, E>
where
    F: Fn() -> I + Send + Clone + 'static, 
    I: IntoProxyEngine<E>,
    E: ProxyEngine<Config = ProxyConfig, Executor = ProxyCommandExecutor>
{
    addresses: Vec<net::SocketAddr>,
    players: Vec<Player>,
    engine: F,
    pub created_time: Instant,
    _i: PhantomData<E>
}

impl<F, I, E> ProxyServer<F, I, E>
where
    F: Fn() -> I + Send + Clone + 'static, 
    I: IntoProxyEngine<E>,
    E: ProxyEngine<Config = ProxyConfig, Executor = ProxyCommandExecutor>
{
    pub fn new(engine: F) -> Self {
        ProxyServer {
            addresses: Vec::new(),
            players: Vec::<Player>::new(),
            created_time: Instant::now(),
            engine: engine,
            _i: PhantomData
        }
    }

    fn to_dyn(&self) -> DynServer {
        DynServer {
            addresses: self.addresses.to_vec(),
            players: self.players.to_vec()
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

    pub fn run(self) -> ProxyServerRunner<F, I, E> {
        if self.addresses.is_empty() {
            panic!("Must be bound to at least one address");
        }

        ProxyServerRunner {
            server: self,
            _i: PhantomData
        }
    }

}

impl Server for DynServer {
    fn get_players(&self) -> Vec<Player> {
        self.players.to_vec()
    }

    fn get_addresses(&self) -> Vec<net::SocketAddr> {
        self.addresses.to_vec()
    }
}

pub struct ProxyServerRunner<F, I, E>
where
    F: Fn() -> I + Send + Clone + 'static, 
    I: IntoProxyEngine<E>,
    E: ProxyEngine<Config = ProxyConfig, Executor = ProxyCommandExecutor>
{
    server: ProxyServer<F, I, E>,
    _i: PhantomData<E>,
}

impl<F, I, E> Future for ProxyServerRunner<F, I, E> 
where
    F: Fn() -> I + Send + Clone + 'static, 
    I: IntoProxyEngine<E>,
    E: ProxyEngine<Config = ProxyConfig, Executor = ProxyCommandExecutor>
{
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_ref();

        let engine = &(self.server.engine);
        let into = crate::engine::into_engine(engine());
        let config = into.get_config().clone();
        let mut commands = into.get_commands();
        let sockets = self.server.addresses.to_vec();
        

        for command in &mut commands {
            command.set_backend(Box::new(self.server.to_dyn())).unwrap();
        }
        
        for socket in sockets {
            tokio::spawn(async move {
                let mut listener = TcpListener::bind(socket).await.unwrap();
                loop {
                    if let Ok(client) = listener.accept().await {
                        client.0.set_nodelay(true).unwrap();
                        let (stream, addr) = client;
                        tokio::spawn(async move {
                            let config = config.clone();
                            match crate::protocol::slp::attempt_server_list_ping(config, stream, addr).await {
                                Ok(handshake) => {
                                    if handshake.next_state == 2 {
                                        info!("handle login");
                                    }
                                },

                                Err(error) => {
                                    error!("{}", error);
                                }
                            } 


                        });
                    }
                }
            });
        }

        tokio::spawn(async move {
            let stdin = io::stdin();
            let mut stdin = io::BufReader::new(stdin);
            loop {
                let mut input = String::new();
                stdin.read_line(&mut input).unwrap();
                input = input.trim().to_owned();
                let split = input.split_ascii_whitespace();
                let cmd = split.clone().next().unwrap().to_lowercase().to_owned();
                
                for command in &commands {
                    if command.get_label().eq(&cmd) ||  command.get_aliases().iter().any(|&i| i.eq(&cmd)) {
                        command.execute(Box::new(ConsoleCommandSender), split.clone().into_iter().skip(1).map(|f| f.to_owned()).collect())
                    } else {
                        println!("Unknown command \"{}\".", &input);
                    }
                }
            }
        });

        info!("Started in {:?}.", &self.server.created_time.elapsed());

        Poll::Pending
    }
}

struct ConsoleCommandSender;
impl CommandSender for ConsoleCommandSender {
    fn get_name(&self) -> &str {
        "Console"
    }

    fn send_message(&self, message: String) {
        info!("{}", message)
    }
}