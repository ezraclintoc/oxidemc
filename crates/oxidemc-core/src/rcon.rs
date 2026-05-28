use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum RconError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Authentication failed — wrong password")]
    AuthFailed,

    #[error("Invalid response from server")]
    InvalidResponse,

    #[error("Not connected")]
    NotConnected,

    #[error("Response was not valid UTF-8")]
    InvalidUtf8,
}

pub struct RconClient {
    stream: TcpStream,
    next_id: i32,
}

fn encode(id: i32, packet_type: i32, body: &str) -> Vec<u8> {
    let length = 4 + 4 + body.len() + 2; // id + type + body + 2 nulls
    let mut packet = Vec::with_capacity(4 + length);
    packet.extend_from_slice(&(length as i32).to_le_bytes());
    packet.extend_from_slice(&id.to_le_bytes());
    packet.extend_from_slice(&packet_type.to_le_bytes());
    packet.extend_from_slice(body.as_bytes());
    packet.extend_from_slice(&[0u8, 0u8]);
    packet
}

async fn read_packet(stream: &mut TcpStream) -> Result<(i32, i32, String), RconError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let length = i32::from_le_bytes(len_buf) as usize;

    if length < 10 {
        return Err(RconError::InvalidResponse);
    }

    let mut buf = vec![0u8; length];
    stream.read_exact(&mut buf).await?;

    let id = i32::from_le_bytes(buf[0..4].try_into().unwrap());
    let packet_type = i32::from_le_bytes(buf[4..8].try_into().unwrap());
    let body = String::from_utf8(buf[8..length - 2].to_vec())
        .map_err(|_| RconError::InvalidUtf8)?;

    Ok((id, packet_type, body))
}

impl RconClient {
    pub async fn connect(host: &str, port: u16, password: &str) -> Result<RconClient, RconError> {
        let mut stream = tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(format!("{}:{}", host, port)),
        )
        .await
        .map_err(|_| RconError::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "connection timed out")))??;

        let auth = encode(1, 3, password);
        stream.write_all(&auth).await?;

        let (id, _, _) = tokio::time::timeout(
            Duration::from_secs(5),
            read_packet(&mut stream),
        )
        .await
        .map_err(|_| RconError::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "auth timed out")))??;

        if id == -1 {
            Err(RconError::AuthFailed)
        } else {
            Ok(RconClient { stream, next_id: 2 })
        }
    }

    pub async fn send_command(&mut self, command: &str) -> Result<String, RconError> {
        let encoded = encode(self.next_id, 2, command);
        self.stream.write_all(&encoded).await?;

        let (_, _, response) = tokio::time::timeout(
            Duration::from_secs(5),
            read_packet(&mut self.stream),
        )
        .await
        .map_err(|_| RconError::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "command timed out")))??;

        self.next_id += 1;
        Ok(response)
    }

    pub async fn disconnect(mut self) -> Result<(), RconError> {
        self.stream.shutdown().await?;
        Ok(())
    }
}
