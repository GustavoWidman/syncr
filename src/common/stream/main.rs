use std::sync::Arc;

use bytes::BytesMut;
use encrypted_pipe::{EncryptedPipe, PipeReader, PipeWriter};
use rand::RngCore;
use rand::rngs::OsRng;
use ring::hkdf::Salt;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};

use anyhow::{anyhow, bail};
use ring::aead::{AES_256_GCM, BoundKey, LessSafeKey, Nonce, OpeningKey};
use ring::rand::{SecureRandom, SystemRandom};
use tokio::sync::Mutex;

use crate::common::packets::nonce::NoncePacket;
use crate::common::{Packet, PacketBase};

pub struct SecureStream {
    pub(crate) reader: PipeReader<EncryptedPipe<OwnedReadHalf>>,
    pub(crate) writer: PipeWriter<EncryptedPipe<OwnedWriteHalf>>,
}

impl SecureStream {
    async fn handshake(stream: &mut TcpStream, password: &str) -> Result<[u8; 12], anyhow::Error> {
        let mut my_nonce = [0u8; 12];
        OsRng.fill_bytes(&mut my_nonce);
        let my_nonce = NoncePacket::build(my_nonce);

        // stream.write_all(&my_salt.to_bytes()).await.unwrap();
        my_nonce.write(stream).await?;

        // header check
        let mut buf = vec![0u8; 4];
        stream.read_exact(&mut buf).await?;
        if buf != b"NONC" {
            bail!("Invalid packet header");
        }

        // packet collection
        let mut buf = NoncePacket::make_buffer()?;
        let n = stream.read_buf(&mut buf).await?;
        let their_nonce = NoncePacket::from_bytes(&buf[..n]);

        if my_nonce.random > their_nonce.random {
            return Ok(my_nonce.nonce);
        } else {
            return Ok(their_nonce.nonce);
        }
    }

    pub async fn new(mut stream: TcpStream) -> Result<Self, anyhow::Error> {
        // todo replace with config or .env reading (maybe read from .syncr dir)
        let pass = "password"; // todo read from config

        // password from str to &[u8, 32]
        let mut buf = [0u8; 32];
        fill_buffer(&mut buf, pass);

        let (reader, writer) = TcpStream::into_split(stream);

        // todo handshake again
        let mut reader = PipeReader::new(
            EncryptedPipe::new(reader, &buf, &[0u8; 12]),
            None,
            Some(4096),
        );
        let mut writer = PipeWriter::new(
            EncryptedPipe::new(writer, &buf, &[0u8; 12]),
            None,
            Some(4096),
        );

        let mut secure_stream = Self { reader, writer };

        Ok(secure_stream)
    }
}

fn fill_buffer(buf: &mut [u8], s: &str) {
    // Zero the buffer first
    buf.fill(0);

    // Copy as much of the string as will fit
    let bytes = s.as_bytes();
    let len = buf.len().min(bytes.len());
    buf[..len].copy_from_slice(&bytes[..len]);
}
