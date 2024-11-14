use std::sync::Arc;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use anyhow::{anyhow, bail};
use ring::aead::{BoundKey, LessSafeKey, Nonce, OpeningKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};

use crate::common::packets::salt::SaltPacket;
use crate::common::Packet;

use super::keys;
use super::read::SecureReader;
use super::AESEngine;

pub struct SecureStream {
    pub(super) stream: TcpStream,
    pub(super) reader: SecureReader,
    engine: AESEngine,
}

impl SecureStream {
    async fn handshake(stream: &mut TcpStream, password: &str) -> Result<AESEngine, anyhow::Error> {
        let my_salt = keys::new_salt();

        stream.write_all(&my_salt.to_bytes()).await.unwrap();

        // header check
        let mut buf = vec![0u8; 4];
        stream.read_exact(&mut buf).await.unwrap();
        if buf != b"SALT" {
            bail!("Invalid packet header");
        }

        // packet collection
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let their_salt = SaltPacket::from_bytes(&buf[..n]);

        if my_salt.random > their_salt.random {
            return Ok(AESEngine::new(password.to_string(), &my_salt.salt)?);
        } else {
            return Ok(AESEngine::new(password.to_string(), &their_salt.salt)?);
        }
    }

    pub async fn new(mut stream: TcpStream) -> Result<Self, anyhow::Error> {
        // todo replace with config or .env reading (maybe read from .syncr dir)
        let pass = "password";

        let mut engine = Self::handshake(&mut stream, pass).await?;

        let mut secure_stream = Self {
            stream,
            engine,
            reader: SecureReader::new(),
        };

        Ok(secure_stream)
    }

    pub(super) async fn recv_buffer(&mut self, buffer: &mut [u8]) -> Result<usize, anyhow::Error> {
        let read_bytes = self.stream.read(buffer).await?;
        if read_bytes == 0 {
            return Ok(0);
        }

        let decrypted_data = self.engine.decrypt_bytes(&buffer[..read_bytes])?;

        buffer[..decrypted_data.len()].copy_from_slice(&decrypted_data);
        Ok(decrypted_data.len())
    }

    pub(super) async fn send_buffer(&mut self, data: &[u8]) -> Result<(), anyhow::Error> {
        let mut data = data.to_owned();
        let encrypted_data = self.engine.encrypt_bytes(&data)?;
        self.stream.write_all(&encrypted_data).await?;
        Ok(())
    }
}
