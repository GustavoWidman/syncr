use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::config::{PARAMS, SECRET};
use super::handlers::Client;
use crate::common::stream::SecureStream;
use crate::model::{self, BlockSizePredictor};
use anyhow::anyhow;
use dirs::home_dir;
use futures::{FutureExt, TryFutureExt};
use rustls::ServerConfig;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::pem::SectionKind;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
pub struct Server {
    listener: TcpListener,
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>,
    predictor: Arc<Mutex<BlockSizePredictor>>,
}

impl Server {
    pub async fn bind(port: u16) -> Result<Self, anyhow::Error> {
        let listener = TcpListener::bind("127.0.0.1:7878").await?;
        println!("Server listening on 127.0.0.1:7878");

        let predictor = model::initialize!("model.json")?;

        Ok(Self {
            listener,
            predictor: Arc::new(Mutex::new(predictor)),
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn accept(&mut self) -> Result<(SecureStream, SocketAddr), anyhow::Error> {
        let (stream, addr) = self.listener.accept().await?;

        let mut stream = SecureStream::new(stream).await?;

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

                    tokio::spawn(async move { cleanup.await });
                }
                Err(e) => {
                    eprintln!("Connection failed: {e}");
                }
            }
        }
    }
}
