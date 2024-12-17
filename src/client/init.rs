use std::sync::Mutex;
use std::thread::park;
use std::{io::Write, sync::Arc, thread};

use dirs::home_dir;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;
use tokio::task;

use super::config::{PARAMS, SECRET};
use crate::common::config::{Config, quick_config};
use crate::common::packets::{DynamicPacket, PacketBase, SanityPacket};
use crate::common::stream::SecureStream;
use crate::model::{self, BlockSizePredictor};

pub struct Client {
    stream: SecureStream,
    config: Config,
    predictor: Mutex<BlockSizePredictor>,
}

impl Client {
    pub async fn connect(config: Option<Config>) -> Result<Self, anyhow::Error> {
        let config = match config {
            Some(c) => c,
            None => quick_config!()?,
        };
        let client_ref = config.as_client()?; // implicitly assert we're in client mode too!

        let stream = TcpStream::connect((
            client_ref.client().server_ip,
            client_ref.client().server_port,
        ))
        .await?;
        let mut stream = SecureStream::new(stream, &config.secret).await?;

        println!("Connected to server");

        let predictor = model::initialize!("model.json")?;

        println!("Initialized predictor model");

        Ok(Self {
            stream,
            config,
            predictor: Mutex::new(predictor),
        })
    }

    pub async fn run(&mut self) {
        let mut buffer = [0; 4096];

        let message = b"hello world";
        let sanity_packet = SanityPacket::build(message.to_vec());
        sanity_packet.write(&mut *self.stream).await.unwrap();

        let message = b"goodbye world";
        let sanity_packet = SanityPacket::build(message.to_vec());
        sanity_packet.write(&mut *self.stream).await.unwrap();

        let mut buf = vec![0; 4096];
        self.stream.read(&mut buf).await.unwrap();

        println!("{}", std::str::from_utf8(&buf).unwrap());
    }
}
