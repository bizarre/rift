use std::{io, net};
use tokio::net::{TcpStream};
use crate::packet::{In, AsyncPacketReadExt, AsyncPacketWriteExt};
use crate::packet::handshake;
use std::io::{Error, ErrorKind};
use crate::util::color::Color;
use crate::player::Player;
use log::{info, debug, error, trace};
use rand::Rng;

pub async fn attempt_login<T: crate::server::Server>(config: crate::config::ProxyConfig, server: &T, stream: &mut TcpStream, addr: net::SocketAddr) -> io::Result<(Player, Vec<u8>)> {
    let req: io::Result<crate::packet::login::Start> = stream.receive().await;
    if let Ok(packet) = req {
        debug!("User \"{}\" initiating login process.", packet.name);
        let token = format!("{}", rand::thread_rng().gen::<i64>());
        let token_bytes = token.as_bytes();

        let encryption_request = crate::packet::login::EncryptionRequest {
            id: String::from(""),
            public_key: server.get_rsa().public_key_to_der().unwrap(),
            token: token_bytes.to_vec()
        };

        trace!("Sent encryption request to {} ({})", packet.name, addr);
        stream.write_packet(encryption_request).await.unwrap();

        let encryption_response_req: io::Result<crate::packet::login::EncryptionResponse> = stream.receive().await;
        if let Ok(encryption_response) = encryption_response_req {
            trace!("Received encryption response from {} ({})", packet.name, addr);

            let decrypted_token = encryption_response.decrypt_token(&server.get_rsa(), token_bytes.len());
            if !std::str::from_utf8(&decrypted_token).unwrap().eq(&token) {
                return Err(Error::new(ErrorKind::Other, "Invalid login token received."));
            }

            let secret = encryption_response.decrypt_secret(&server.get_rsa());

            let resp = reqwest::get(&format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",
            packet.name,
            crate::util::hash::server_hash("", &secret, &server.get_rsa().public_key_to_der().unwrap())))
                .await.unwrap()
                .json::<Player>()
                .await.unwrap();

            trace!("Authenticated {} ({})", packet.name, addr);

            return Ok((resp, secret))
        } else {
            return Err(Error::new(ErrorKind::Other, "Invalid encryption response."));
        }
    } else {
        return Err(Error::new(ErrorKind::Other, "Invalid login process initiation."));
    }
}