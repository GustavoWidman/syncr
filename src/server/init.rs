use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::handlers::Client;
use crate::common::config::Config;
use crate::common::quick_config;
use crate::common::stream::SecureStream;
use crate::data::DatabaseDriver;
use crate::model::{self, CompressionTree};
use crate::server::database::ServerDatabase;
use futures::FutureExt;
use log::info;
use tokio::net::TcpListener;
pub struct Server {
    listener: TcpListener,
    config: Config,
    database: ServerDatabase,
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>,
    predictor: Arc<Mutex<CompressionTree>>,
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
        info!(
            "Server listening on {}:{}",
            server_ref.server().ip,
            server_ref.server().port
        );

        let mut database = ServerDatabase::new(None).await?;

        info!("Connected to database");

        let predictor = model::initialize!(&mut database)?;

        info!("Initialized predictor model");

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

        let stream = SecureStream::new(stream, &self.config.secret).await?;

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
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr.to_string());

                    let handle = tokio::spawn(async move { Client::handle(stream).await });

                    match self.insert_client(addr, Client::new(handle.abort_handle())) {
                        Ok(_) => info!("Client inserted"),
                        Err(e) => {
                            log::error!("Client insertion failed: {e}");
                            handle.abort();
                            info!("{:?}", self.clients.lock().unwrap().len());
                            info!("Client aborted");
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
                            Ok(_) => info!("Client disconnected"),
                            Err(e) => log::error!("Client disconnected: {e}"),
                        }

                        // print all clients
                        info!("Clients: {:?}", clients.lock().unwrap().len());
                    });

                    tokio::spawn(cleanup);
                }
                Err(e) => {
                    log::error!("Connection failed: {e}");
                }
            }
        }
    }
}
