use std::sync::Mutex;
use std::{io::Write, sync::Arc, thread};

use dirs::home_dir;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, TlsConnector};

use super::config::{PARAMS, SECRET};
use crate::common::crypto::SecureStream;
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
        loop {
            match self.stream.read(&mut buffer).await {
                Ok(0) => {
                    println!("Server disconnected");
                    break;
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
