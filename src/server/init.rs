use sea_orm::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::handlers::Client;
use crate::common::config::Config;
use crate::common::config::structure::ModeConfig;
use crate::common::quick_config;
use crate::common::stream::SecureStream;
use crate::data::DatabaseDriver;
use crate::data::entities::predictor::Entity as PredictorModel;
use crate::model::{self, BlockSizePredictor};
use crate::server::database::ServerDatabase;
use anyhow::{anyhow, bail};
use futures::FutureExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
pub struct Server {
    listener: TcpListener,
    config: Config,
    database: ServerDatabase,
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>,
    predictor: Arc<Mutex<BlockSizePredictor>>,
}

impl Server {
    pub async fn bind(config: Option<Config>) -> Result<Self, anyhow::Error> {
        let config = match config {
            Some(c) => c,
            None => quick_config!()?,
        };
        let server_ref = config.as_server()?; // implicitly assert we're in server mode too!

        let listener = TcpListener::bind((
            server_ref.server().ip, //
            server_ref.server().port,
        ))
        .await?;
        println!(
            "Server listening on {}:{}",
            server_ref.server().ip,
            server_ref.server().port
        );

        let database = ServerDatabase::new(None).await?;

        let model = PredictorModel::find_by_id(1).one(&*database).await?;

        let predictor = model::BlockSizePredictor::rescue(model)?;

        println!("Initialized predictor model");

        Ok(Self {
            database,
            listener,
            config,
            predictor: Arc::new(Mutex::new(predictor)),
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn accept(&mut self) -> Result<(SecureStream, SocketAddr), anyhow::Error> {
        let (stream, addr) = self.listener.accept().await?;

        let mut stream = SecureStream::new(stream, &self.config.secret).await?;

        Ok((stream, addr))
    }

    fn insert_client(&self, addr: SocketAddr, client: Client) -> Result<(), anyhow::Error> {
        if self
            .clients
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .insert(addr, client)
            .is_some()
        {
            return Err(anyhow::anyhow!("Client already exists"));
        }

        Ok(())
    }

    pub async fn run(&mut self) {
        loop {
            match self.accept().await {
                Ok((mut stream, addr)) => {
                    println!("New connection from {}", addr.to_string());

                    let handle = tokio::spawn(async move { Client::handle(stream).await });

                    match self.insert_client(addr, Client::new(handle.abort_handle())) {
                        Ok(_) => println!("Client inserted"),
                        Err(e) => {
                            eprintln!("Client insertion failed: {e}");
                            handle.abort();
                            println!("{:?}", self.clients.lock().unwrap().len());
                            println!("Client aborted");
                            continue;
                        }
                    }

                    let clients = self.clients.clone();
                    let addr = Arc::new(addr);
                    let cleanup = handle.then(|result| async move {
                        // move clone of clients in

                        // todo remove unwraps
                        clients.lock().unwrap().remove(&addr).unwrap();

                        match result {
                            Ok(_) => println!("Client disconnected"),
                            Err(e) => eprintln!("Client disconnected: {e}"),
                        }

                        // print all clients
                        println!("Clients: {:?}", clients.lock().unwrap().len());
                    });

                    tokio::spawn(cleanup);
                }
                Err(e) => {
                    eprintln!("Connection failed: {e}");
                }
            }
        }
    }
}
