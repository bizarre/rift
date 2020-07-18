use crate::packet::{In, Out, AsyncPacketReadExt, AsyncPacketWriteExt};
use crate::packet;
use async_trait::async_trait;
use std::io::{Error, ErrorKind};
use crate::player::Player;
use serde::Serialize;

#[derive(Debug)]
pub struct Packet {
    pub version: i32,
    pub address: String,
    pub port: u16,
    pub next_state: i32
}

pub struct Request;

pub struct Ping {
    pub _fluff: i64
}

#[derive(Serialize)]
pub struct Response {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>
}

#[derive(Serialize)]
pub struct Description {
    pub text: String
}

#[derive(Serialize)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<Player>
}

#[derive(Serialize)]
pub struct Version {
    pub name: String,
    pub protocol: i32
}


impl packet::Packet for Packet {
    fn get_id(&self) -> i32 {
        0
    }
}

#[async_trait]
impl In for Packet {
    async fn read<R: AsyncPacketReadExt + std::marker::Unpin + Send + Sync>(buffer: &mut R) -> std::io::Result<Self> where Self: Sized {
        let _ = buffer.read_varint().await?; //todo: maybe offload this kind of logic to calling function
        let id = buffer.read_varint().await?;

        if id != 0x00 {
            return Err(Error::new(ErrorKind::Other, "Invalid handshake packet."));
        }
        
        let version: i32 = buffer.read_varint().await?;
        let address = buffer.read_string().await?;
        let port = buffer.read_u16().await?;
        let next_state: i32 = buffer.read_varint().await?;

        Ok(Packet {
            version: version,
            address: address,
            port: port,
            next_state: next_state
        })
    }
}

impl packet::Packet for Ping {
    fn get_id(&self) -> i32 {
        1
    }
}

#[async_trait]
impl In for Ping {
    async fn read<R: AsyncPacketReadExt + std::marker::Unpin + Send + Sync>(buffer: &mut R) -> std::io::Result<Self> where Self: Sized {
        let _ = buffer.read_varint().await?; //todo: maybe offload this kind of logic to calling function
        let id = buffer.read_varint().await?;

        if id != 0x01 {
            return Err(Error::new(ErrorKind::Other, "Invalid ping packet."));
        }
        
        Ok(Ping {
            _fluff: buffer.read_long().await?
        })
    }
}


#[async_trait]
impl Out for Ping {
    async fn write<W: AsyncPacketWriteExt + std::marker::Unpin + Send + Sync>(self, buffer: &mut W) -> std::io::Result<()> {
        buffer.write_long(self._fluff).await?;
        Ok(())
    }
}


impl packet::Packet for Request {
    fn get_id(&self) -> i32 {
        0
    }
}

#[async_trait]
impl In for Request {
    async fn read<R: AsyncPacketReadExt + std::marker::Unpin + Send + Sync>(buffer: &mut R) -> std::io::Result<Self> where Self: Sized {
        let _ = buffer.read_varint().await?; //todo: maybe offload this kind of logic to calling function
        let id = buffer.read_varint().await?;

        if id != 0x00 {
            return Err(Error::new(ErrorKind::Other, "Invalid request packet."));
        }
        
        Ok(Request)
    }
}

impl packet::Packet for Response {
    fn get_id(&self) -> i32 {
        0
    }
}


#[async_trait]
impl Out for Response {
    async fn write<W: AsyncPacketWriteExt + std::marker::Unpin + Send + Sync>(self, buffer: &mut W) -> std::io::Result<()> {
        buffer.write_string(serde_json::to_string(&self).unwrap()).await?;
        Ok(())
    }
}