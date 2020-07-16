pub mod handshake;

use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io::Result;
use async_trait::async_trait;

pub trait Packet {
 fn get_id(&self) -> i32;
}

#[async_trait]
pub trait Out : Packet {
    async fn write<W: AsyncPacketWriteExt + std::marker::Unpin + Send + Sync>(self, buffer: &mut W) -> Result<()>;
}
#[async_trait]
pub trait In : Packet {
    async fn read<R: AsyncPacketReadExt + std::marker::Unpin + Send + Sync>(buffer: &mut R) -> Result<Self> where Self: Sized;
}

#[async_trait]
pub trait AsyncPacketWriteExt : AsyncWriteExt {
    async fn write_varint(&mut self, value: i32) -> Result<()>;
    async fn write_long(&mut self, value: i64) -> Result<()>;
    async fn write_string(&mut self, value: String) -> Result<()>;
    async fn write_ushort(&mut self, value: u16) -> Result<()>;
    async fn write_packet<T: Packet + Out + Send + Sync>(&mut self, packet: T) -> Result<()>;
}

#[async_trait]
pub trait AsyncPacketReadExt : AsyncReadExt {
    async fn read_varint(&mut self) -> Result<i32>;
    async fn read_string(&mut self) -> Result<String>;
    async fn read_long(&mut self) -> Result<i64>;
    async fn read_ushort(&mut self) -> Result<u16>;
    async fn receive<T: Packet + In + Send + Sync>(&mut self) -> Result<T>;
}

#[async_trait]
impl<W: AsyncWrite + Unpin + Send + Sync> AsyncPacketWriteExt for W {
    async fn write_varint(&mut self, mut value: i32) -> Result<()> {
        let mut buffer = [0; 5]; // VarInts are never longer than 5 bytes
        let mut counter = 0;

        loop {
            let mut temp = (value & 0b01111111) as u8;

            value >>= 7;
            if value != 0 {
                temp |= 0b10000000;
            }

            buffer[counter] = temp;

            counter += 1;

            if value == 0 {
                break;
            }
        }

        self.write_all(&mut buffer[0..counter]).await?;

        Ok(())
    }


    async fn write_long(&mut self, value: i64) -> Result<()> {
        self.write_all(&value.to_be_bytes()).await?;

        Ok(())
    }

    async fn write_string(&mut self, value: String) -> Result<()> {
        self.write_varint(value.len() as i32).await?;
        self.write_all(value.as_bytes()).await?;

        Ok(())
    }

    async fn write_ushort(&mut self, value: u16) -> Result<()> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_packet<T: Packet + Out + Send + Sync>(&mut self, packet: T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut payload = Vec::new();

        buffer.write_varint(packet.get_id()).await?;
        packet.write(&mut buffer).await?;

        payload.write_varint(buffer.len() as i32).await?;
        payload.write_all(&buffer).await?;

        self.write_all(&payload).await?;

        Ok(())
    }
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send + Sync> AsyncPacketReadExt for R {
    async fn read_varint(&mut self) -> Result<i32> {
        let mut buffer = [0];
        let mut counter = 0;
        let mut value = 0;

        loop {
            self.read_exact(&mut buffer).await?;

            let temp = (buffer[0] as i32) & 0b01111111;

            value |= temp << (counter * 7);
            counter += 1;

            if counter > 5 {
                panic!("invalid data");
            }

            if buffer[0] & 0b10000000 == 0 {
                break;
            }
        }

        Ok(value)
    }

    async fn read_long(&mut self) -> Result<i64> {
        let mut buffer = [0; 8];
        self.read_exact(&mut buffer).await?;

        let result: i64 = i64::from_be_bytes(buffer);

        Ok(result)
    }

    async fn read_ushort(&mut self) -> Result<u16> {
        let mut buffer = [0; 2];
        self.read_exact(&mut buffer).await?;

        let result: u16 = u16::from_be_bytes(buffer);

        Ok(result)
    }


    async fn read_string(&mut self) -> Result<String> {
        let size = self.read_varint().await?;
        let mut buffer = vec![0; size as usize];
        
        self.read_exact(&mut buffer).await?;
        
        let string = String::from_utf8(buffer).unwrap();

        return Ok(string)
    }

    async fn receive<T: Packet + In + Send + Sync>(&mut self) -> Result<T> {
        let mut time: i32 = 0;

        loop {
            let packet = T::read(self).await;
            if packet.is_ok() {
                return Ok(packet.unwrap());
            }

            time += 1;
            tokio::time::delay_for(tokio::time::Duration::from_secs(1)).await;

            if time > 3 {
                panic!("Request timed out.");
            }
        }
    }
}