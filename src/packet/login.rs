use crate::packet::{In, Out, AsyncPacketReadExt, AsyncPacketWriteExt};
use crate::packet::{Packet};
use async_trait::async_trait;
use std::io::{Error, ErrorKind};
use crate::player::Player;
use serde::Serialize;

#[derive(Debug)]
pub struct Start {
    pub name: String
}


impl Packet for Start {
    fn get_id(&self) -> i32 {
        0
    }
}

#[async_trait]
impl In for Start {
    async fn read<R: AsyncPacketReadExt + std::marker::Unpin + Send + Sync>(buffer: &mut R) -> std::io::Result<Self> where Self: Sized {
        let _ = buffer.read_varint().await?; //todo: maybe offload this kind of logic to calling function
        let id = buffer.read_varint().await?;

        if id != 0x00 {
            return Err(Error::new(ErrorKind::Other, "Invalid login start packet."));
        }
        
        Ok(Start {
            name: buffer.read_string().await?
        })
    }
}

#[derive(Debug)]
pub struct EncryptionRequest {
    pub id: String,
    pub public_key: Vec<u8>,
    pub token: Vec<u8>
}


impl Packet for EncryptionRequest {
    fn get_id(&self) -> i32 {
        1
    }
}


#[async_trait]
impl Out for EncryptionRequest {
    async fn write<W: AsyncPacketWriteExt + std::marker::Unpin + Send + Sync>(self, buffer: &mut W) -> std::io::Result<()> {
        buffer.write_string(self.id).await?;
        buffer.write_varint(self.public_key.len() as i32).await?;
        buffer.write(&self.public_key).await?;
        buffer.write_varint(self.token.len() as i32).await?;
        buffer.write(&self.token).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EncryptionResponse {
    pub secret: [u8; 16],
    pub token: Vec<u8>
}

impl Packet for EncryptionResponse {
    fn get_id(&self) -> i32 {
        1
    }
}


#[async_trait]
impl In for EncryptionResponse {
    async fn read<R: AsyncPacketReadExt + std::marker::Unpin + Send + Sync>(buffer: &mut R) -> std::io::Result<Self> where Self: Sized {
        let _ = buffer.read_varint().await?; //todo: maybe offload this kind of logic to calling function
        let id = buffer.read_varint().await?;

        if id != 0x01 {
            return Err(Error::new(ErrorKind::Other, "Invalid encryption response packet."));
        }

        let mut secret: [u8; 16] = [0; 16];
        buffer.read_exact(&mut secret).await?;

        let mut token = vec![0; buffer.read_varint().await? as usize];
        buffer.read_exact(&mut token).await?;
        
        Ok(EncryptionResponse {
            secret: secret,
            token: token.to_vec()
        })
    }
}

