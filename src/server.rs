use std::{io, net};
use std::fmt::Display;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::BufRead;
use tokio::time::{Instant};
use log::{info, debug, error, trace};
use crate::command::{CommandSender, ProxyCommandExecutor};
use crate::player::Player;
use crate::engine::{ProxyEngine, IntoProxyEngine};
use crate::config::{ProxyConfig, ServerConfig};
use std::marker::PhantomData;
use crate::packet::{In, Out, AsyncPacketReadExt, AsyncPacketWriteExt};
use openssl::rsa::{Rsa, Padding};
use rand::Rng;
use crate::packet::Chat;
use std::error::Error;

pub trait Server {
    fn get_players(&self) -> Vec<Player>;
    fn get_addresses(&self) -> Vec<net::SocketAddr>;
    fn get_rsa(&self) -> Rsa<openssl::pkey::Private>;
}

#[derive(Clone)]
struct DynServer {
    players: Vec<Player>,
    addresses: Vec<net::SocketAddr>,
    rsa: Rsa<openssl::pkey::Private>
}

pub struct ProxyServer<F, I, E>
where
    F: Fn() -> I + Send + Clone + 'static, 
    I: IntoProxyEngine<E>,
    E: ProxyEngine<Config = ProxyConfig, Executor = ProxyCommandExecutor>
{
    rsa: Rsa<openssl::pkey::Private>,
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
            rsa: Rsa::generate(1028).unwrap(),
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
            players: self.players.to_vec(),
            rsa: self.rsa.clone()
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

    fn get_rsa(&self) -> Rsa<openssl::pkey::Private> {
        self.rsa.clone()
    }
}

pub struct ProxyServerRunner<F, I, E>
where
    F: Fn() -> I + Send + Clone + 'static, 
    I: IntoProxyEngine<E>,
    E: ProxyEngine<Config = ProxyConfig, Executor = ProxyCommandExecutor>
{
    pub server: ProxyServer<F, I, E>,
    _i: PhantomData<E>,
}

impl<F, I, E> Future for ProxyServerRunner<F, I, E> 
where
    F: Fn() -> I + Send + Clone + 'static + Unpin, 
    I: IntoProxyEngine<E>,
    E: ProxyEngine<Config = ProxyConfig, Executor = ProxyCommandExecutor> + 'static + Unpin
{
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let engine = &(this.server.engine);
        let into = crate::engine::into_engine(engine());
        let config = into.get_config().clone();
        let mut commands = into.get_commands();
        let sockets = this.server.addresses.to_vec();

        for command in &mut commands {
            command.set_backend(Box::new(this.server.to_dyn())).unwrap();
        }
        
        for socket in sockets {
            let server = this.server.to_dyn();
            let config = config.clone();
            tokio::spawn(async move {
                let mut listener = TcpListener::bind(socket).await.unwrap();
                let cloned = server.clone();
                let config = config.clone();
                loop {
                    let cloned = cloned.clone();
                    let config = config.clone();
                    if let Ok(client) = listener.accept().await {
                        client.0.set_nodelay(true).unwrap();
                        let (mut stream, addr) = client;
                        tokio::spawn(async move {
                            let config = config.clone();

                            match crate::protocol::slp::attempt_server_list_ping(config.clone(), &cloned, &mut stream, addr).await {
                                Ok(handshake) => {
                                    if handshake.next_state == 2 {
                                        if let Ok(default_server) = config.get_default_server() {
                                            match crate::protocol::login::attempt_login(config.clone(), &cloned, &mut stream, addr).await {
                                                Ok((player, secret)) => {
                                                    
                                                   let target = default_server.get_address();
                                                   if let Ok(mut server) = TcpStream::connect(target).await {
                                                       trace!("Established proxy connection to {} ({}) for {}.", default_server.id, target, player.name);
                                                       server.set_nodelay(true);
                                                       server.write_packet(handshake.clone()).await.unwrap();
                                                       server.write_packet(crate::packet::login::Start {
                                                           name: player.name.to_owned()
                                                       }).await.unwrap();

                                                       let success_packet: io::Result<crate::packet::login::Success> = server.receive().await;
                                                       if success_packet.is_ok() {
                                                           let success_packet = crate::packet::login::Success {
                                                               uuid: player.id.to_string(),
                                                               name: player.name.to_owned()
                                                           };

                                                           info!("Got login success packet!");
                                                       } else {
                                                           let error = format!("{}", success_packet.err().unwrap());
                                                           if let Ok(packet_id) = error.parse::<i32>() {
                                                                if packet_id == 0 { // disconnect packet
                                                                    let message = server.read_string().await.unwrap();
                                                                    let message: Chat = serde_json::from_str(&message).unwrap();

                                                                    if let Some(translate) = &message.translate {
                                                                        if translate.contains("Connection throttled") {
                                                                            error!("Connection throttle is enabled for {}. Turn it off!", default_server.id);
                                                                        }   
                                                                    }

                                                                    stream.write_packet_encrypted(crate::packet::login::Disconnect {
                                                                            chat: message // directly proxy the disconnect message from destination server
                                                                    }, &secret).await.unwrap();
                                                                    return
                                                                }
                                                            }

                                                            error!("Failed to connect {} to {}!", player.name, default_server.id);
                                                       }

                                                   } else {
                                                       trace!("Failed to connect {} to {}!", player.name, default_server.id);
                                                       stream.write_packet_encrypted(crate::packet::login::Disconnect {
                                                            chat: Chat::new(format!("&cFailed to connect to {}!", default_server.id))
                                                       }, &secret).await.unwrap();
                                                   }

                                                   /*stream.write_packet_encrypted(crate::packet::login::Success {
                                                    player: resp.clone()
                                                    }, &secret).await.unwrap();*/
                                                },
                
                                                Err(error) => {
                                                    error!("{}", error);
                                                }
                                            }
                                        } else {
                                            stream.write_packet(crate::packet::login::Disconnect {
                                                chat: Chat::new("&cWe don't know where to send you!")
                                            }).await.unwrap();

                                            error!("No default server defined, we don't know where to send player!");
                                        }
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

        info!("Started in {:?}.", &this.server.created_time.elapsed());

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