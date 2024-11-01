use std::{net::TcpListener, thread};

use super::config::{PARAMS, SECRET};
use super::handlers::Client;
use crate::common::noise::{handshake_server, Transport};

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
}

impl Server {
    pub fn bind(port: u16) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("Failed to bind");

        println!("Listening on http://127.0.0.1:{port}");

        Self {
            listener,
            clients: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("New connection from {}", stream.peer_addr().unwrap());
                    let noise = handshake_server!(SECRET, PARAMS, &mut stream).unwrap();

                    let transport = Transport::new(stream, noise);
                    let handle = thread::spawn(move || {
                        Client::handle(transport);
                    });
                    self.clients.push(Client::new(handle))
                }
                Err(e) => {
                    eprintln!("Connection failed: {e}");
                }
            }
        }
    }
}
