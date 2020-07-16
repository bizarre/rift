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
use std::io::{Error, ErrorKind};

pub async fn attempt_server_list_ping(config: crate::config::ProxyConfig, mut stream: TcpStream, addr: net::SocketAddr) -> io::Result<handshake::Packet> {
    if let Ok(handshake) = crate::packet::handshake::Packet::read(&mut stream).await {
        if handshake.next_state == 2 {
            return Ok(handshake)
        }

        let req: io::Result<crate::packet::handshake::Request> = stream.receive().await;
        if req.is_ok() {
            info!("Client ({}) initiated handshake to proxy via {}.", addr, handshake.address);
            let response = handshake::Response {
                players: handshake::Players {
                    max: 1,
                    online: 100,
                    sample: Vec::new()
                },
                description: handshake::Description {
                    text: String::from("Rift Baby!")
                },
                version: handshake::Version {
                    name: String::from("Rift"),
                    protocol: handshake.version
                }
            };
            

            stream.write_packet(response).await.unwrap();

            if let Ok(ping) = crate::packet::handshake::Ping::read(&mut stream).await {
                stream.write_packet(ping).await.unwrap();
                Ok(handshake)
            } else {
                Err(Error::new(ErrorKind::Other, "Bad ping packet."))
            }
        } else {
            Err(Error::new(ErrorKind::Other, "Bad request packet."))
        }
    } else {
        error!("Malformed handshake packet from {}!", addr);
        return Err(Error::new(ErrorKind::Other, "Invalid handshake packet."));
    }

}