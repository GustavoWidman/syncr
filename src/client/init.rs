use std::sync::Mutex;
use std::thread::park;
use std::{io::Write, sync::Arc, thread};

use dirs::home_dir;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;
use tokio::task;
use tokio_rustls::{TlsConnector, client::TlsStream};

use super::config::{PARAMS, SECRET};
use crate::common::packets::sanity::SanityPacket;
use crate::common::packets::size;
use crate::common::stream::SecureStream;
use crate::common::{DynamicPacket, Packet, PacketBase};
use crate::model::{self, BlockSizePredictor};

pub struct Client {
    stream: SecureStream,
    predictor: Mutex<BlockSizePredictor>,
}

impl Client {
    pub async fn connect(address: &str) -> Result<Self, anyhow::Error> {
        let stream = TcpStream::connect("127.0.0.1:7878").await?;
        let mut stream = SecureStream::new(stream).await?;

        println!("Connected to server");

        let predictor = model::initialize!("model.json")?;

        println!("Initialized predictor model");

        Ok(Self {
            stream,
            predictor: Mutex::new(predictor),
        })
    }

    pub async fn run(&mut self) {
        let mut buffer = [0; 4096];
        // send a salt packet (static size)

        // let salt_packet = SaltPacket::default();
        // salt_packet.write(stream.clone()).await.unwrap();

        // // send a sanity packet (dynamic size)
        let message = b"hello world";
        let sanity_packet = SanityPacket::build(message.to_vec());
        sanity_packet.write(&mut self.stream.writer).await.unwrap();

        let message = b"goodbye world";
        let sanity_packet = SanityPacket::build(message.to_vec());
        sanity_packet.write(&mut self.stream.writer).await.unwrap();

        let mut buf = vec![0; 4096];
        self.stream.reader.read(&mut buf).await.unwrap();

        println!("{}", std::str::from_utf8(&buf).unwrap());
    }
}
