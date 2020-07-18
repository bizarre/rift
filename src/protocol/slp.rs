use std::{io, net};
use tokio::net::{TcpStream};
use log::{info, error};
use crate::packet::{In, AsyncPacketReadExt, AsyncPacketWriteExt};
use crate::packet::handshake;
use std::io::{Error, ErrorKind};
use crate::util::color::Color;

pub async fn attempt_server_list_ping<T: crate::server::Server>(config: crate::config::ProxyConfig, server: T, mut stream: TcpStream, addr: net::SocketAddr) -> io::Result<handshake::Packet> {
    if let Ok(handshake) = crate::packet::handshake::Packet::read(&mut stream).await {
        if handshake.next_state == 2 {
            return Ok(handshake)
        }

        let req: io::Result<crate::packet::handshake::Request> = stream.receive().await;
        if req.is_ok() {
            info!("Client ({}) initiated handshake to proxy via {}.", addr, handshake.address);
            let mut response = handshake::Response {
                players: handshake::Players {
                    max: config.max_players,
                    online: server.get_players().len() as i32,
                    sample: server.get_players()
                },
                description: handshake::Description {
                    text: config.motd.to_owned().colored()
                },
                version: handshake::Version {
                    name: String::from("Rift"),
                    protocol: handshake.version
                },
                favicon: None
            };

            if config.favicon.is_some() {
                response.favicon = Some(config.favicon.unwrap().to_owned());
            }

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