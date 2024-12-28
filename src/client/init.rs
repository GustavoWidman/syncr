use std::sync::Mutex;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::common::config::{Config, quick_config};
use crate::common::packets::{DynamicPacket, PacketBase, SanityPacket};
use crate::common::stream::SecureStream;
use crate::data::DatabaseDriver;
use crate::model::{self, CompressionTree};

use super::database::ClientDatabase;

pub struct Client {
    stream: SecureStream,
    config: Config,
    database: ClientDatabase,
    predictor: Mutex<CompressionTree>,
}

impl Client {
    pub async fn connect(config: Option<Config>) -> Result<Self, anyhow::Error> {
        let config = match config {
            Some(c) => c,
            None => quick_config!()?,
        };
        let client_ref = config.as_client()?; // implicitly assert we're in client mode too!

        let mut database = ClientDatabase::new(None).await?;

        println!("Connected to database");

        let predictor = model::initialize!(&mut database)?;

        println!("Initialized predictor model");

        let stream = TcpStream::connect((
            client_ref.client().server_ip,
            client_ref.client().server_port,
        ))
        .await?;
        let stream = SecureStream::new(stream, &config.secret).await?;

        println!("Connected to server");

        Ok(Self {
            stream,
            config,
            predictor: Mutex::new(predictor),
            database,
        })
    }

    pub async fn run(&mut self) {
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
