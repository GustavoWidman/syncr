use std::sync::Arc;

use bytes::BytesMut;
use ring::hkdf::Salt;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use anyhow::{anyhow, bail};
use ring::aead::{BoundKey, LessSafeKey, Nonce, OpeningKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use tokio::sync::Mutex;

use crate::common::packets::salt::SaltPacket;
use crate::common::Packet;

use super::keys;
use super::read::SecureReader;
use super::AESEngine;

pub struct SecureStream {
    pub(super) stream: Arc<Mutex<TcpStream>>,
    pub(super) reader: SecureReader,
    engine: AESEngine,
}

impl SecureStream {
    async fn handshake(
        stream: Arc<Mutex<TcpStream>>,
        password: &str,
    ) -> Result<AESEngine, anyhow::Error> {
        let my_salt = keys::new_salt().arc();

        // stream.write_all(&my_salt.to_bytes()).await.unwrap();
        my_salt.clone().write(stream.clone()).await?;

        let mut stream = stream.lock().await;

        // header check
        let mut buf = vec![0u8; 4];
        stream.read_exact(&mut buf).await?;
        if buf != b"SALT" {
            bail!("Invalid packet header");
        }

        // packet collection
        let mut buf = SaltPacket::make_buffer(&None)?;
        let n = stream.read_buf(&mut buf).await?;
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

        let arcutex_stream = Arc::new(Mutex::new(stream));

        let mut engine = Self::handshake(arcutex_stream.clone(), pass).await?;

        let mut secure_stream = Self {
            stream: arcutex_stream,
            engine,
            reader: SecureReader::new(),
        };

        Ok(secure_stream)
    }

    pub(super) async fn recv_buffer(&mut self, buffer: &mut [u8]) -> Result<usize, anyhow::Error> {
        let mut stream = self.stream.lock().await;

        let read_bytes = stream.read(buffer).await?;
        if read_bytes == 0 {
            return Ok(0);
        }

        let decrypted_data = self.engine.decrypt_bytes(&buffer[..read_bytes])?;

        buffer[..decrypted_data.len()].copy_from_slice(&decrypted_data);
        Ok(decrypted_data.len())
    }

    pub(super) async fn send_buffer(&mut self, data: &[u8]) -> Result<(), anyhow::Error> {
        let mut stream = self.stream.lock().await;

        let mut data = data.to_owned();
        let encrypted_data = self.engine.encrypt_bytes(&data)?;
        stream.write_all(&encrypted_data).await?;
        Ok(())
    }
}
