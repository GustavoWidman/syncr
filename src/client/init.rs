use std::sync::Mutex;
use std::{io::Write, sync::Arc, thread};

use dirs::home_dir;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;
use tokio_rustls::{client::TlsStream, TlsConnector};

use super::config::{PARAMS, SECRET};
use crate::common::crypto::SecureStream;
use crate::common::packets::sanity::SanityPacket;
use crate::common::packets::size;
use crate::common::Packet;
use crate::model::{self, BlockSizePredictor};

pub struct Client {
    stream: Arc<TokioMutex<SecureStream>>,
    predictor: Mutex<BlockSizePredictor>,
}

impl Client {
    pub async fn connect(address: &str) -> Result<Self, anyhow::Error> {
        let stream = TcpStream::connect("127.0.0.1:7878").await?;
        let mut stream = Arc::new(TokioMutex::new(SecureStream::new(stream).await?));

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
        let mut stream = self.stream.clone();
        // send a sanity packet
        let message = b"hello world";
        let sanity_packet = SanityPacket::build(message.to_vec()).arc();
        sanity_packet.clone().write(stream.clone()).await.unwrap();

        loop {
            match stream.lock().await.read(&mut buffer).await {
                Ok(0) => {
                    // println!("Server disconnected");
                    // break;
                    continue;
                }
                Ok(n) => {
                    println!("{}", std::str::from_utf8(&buffer[..n]).unwrap());

                    buffer = [0; 4096]; // Clear buffer
                }
                Err(e) => {
                    eprintln!("Failed to read from stream: {:?}", e);
                    break;
                }
            };
        }
    }
}
