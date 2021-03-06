use crate::packet::{In, Out, AsyncPacketReadExt, AsyncPacketWriteExt};
use crate::packet::{Packet};
use async_trait::async_trait;
use std::io::{Error, ErrorKind};
use crate::player::Player;
use serde::Serialize;
use openssl::rsa::{Rsa, Padding};
use serde_json;

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

#[async_trait]
impl Out for Start {
    async fn write<W: AsyncPacketWriteExt + std::marker::Unpin + Send + Sync>(self, buffer: &mut W) -> std::io::Result<()> {
        buffer.write_string(self.name).await?;
        Ok(())
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
    pub secret: Vec<u8>,
    pub token: Vec<u8>
}

impl Packet for EncryptionResponse {
    fn get_id(&self) -> i32 {
        1
    }
}

impl EncryptionResponse {
    pub fn decrypt_token(&self, key: &Rsa<openssl::pkey::Private>, length: usize) -> Vec<u8> {
        let mut to_return: Vec<u8> = vec![0; self.token.len() as usize];
        let read = key.private_decrypt(&self.token, &mut to_return, Padding::PKCS1).unwrap();

        to_return[..length].to_vec()
    }

    pub fn decrypt_secret(&self, key: &Rsa<openssl::pkey::Private>) -> Vec<u8> {
        let mut to_return: Vec<u8> = vec![0; self.secret.len() as usize];
        let read = key.private_decrypt(&self.secret, &mut to_return, Padding::PKCS1).unwrap();

        to_return[..16].to_vec()
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

        let mut secret = vec![0; buffer.read_varint().await? as usize];
        buffer.read_exact(&mut secret).await?;

        let mut token = vec![0; buffer.read_varint().await? as usize];
        buffer.read_exact(&mut token).await?;
        
        Ok(EncryptionResponse {
            secret: secret,
            token: token.to_vec()
        })
    }
}


#[derive(Debug)]
pub struct Success {
    pub uuid: String,
    pub name: String
}


impl Packet for Success {
    fn get_id(&self) -> i32 {
        2
    }
}


#[async_trait]
impl Out for Success {
    async fn write<W: AsyncPacketWriteExt + std::marker::Unpin + Send + Sync>(self, buffer: &mut W) -> std::io::Result<()> {
        buffer.write_string(self.uuid).await?;
        buffer.write_string(self.name).await?;
        Ok(())
    }
}

#[async_trait]
impl In for Success {
    async fn read<R: AsyncPacketReadExt + std::marker::Unpin + Send + Sync>(buffer: &mut R) -> std::io::Result<Self> where Self: Sized {
        let size = buffer.read_varint().await?; //todo: maybe offload this kind of logic to calling function
        let id = buffer.read_varint().await?;

        if id != 0x02 {
            return Err(Error::new(ErrorKind::Other, format!("{}", id)));
        }

        // im assuming offline mode servers dont send back a uuid or name in this packet ?

        Ok(Success {
            uuid: String::from("N/A"),
            name: String::from("N/A")
        })
    }
}

#[derive(Debug)]
pub struct Disconnect {
    pub chat: crate::packet::Chat
}


impl Packet for Disconnect {
    fn get_id(&self) -> i32 {
        0
    }
}


#[async_trait]
impl Out for Disconnect {
    async fn write<W: AsyncPacketWriteExt + std::marker::Unpin + Send + Sync>(self, buffer: &mut W) -> std::io::Result<()> {
        buffer.write_string(serde_json::to_string(&self.chat)?).await?;
        Ok(())
    }
}


