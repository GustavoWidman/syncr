use std::{io::Write, net::TcpStream, thread};

use super::config::{PARAMS, SECRET};
// use super::handlers::Client;
use crate::common::{
    noise::{handshake_client, Transport},
    packets::auth::AuthenticationPacket,
    Packet,
};

pub struct Client {
    transport: Transport,
}

impl Client {
    pub fn connect(address: &str) -> Self {
        let mut stream = TcpStream::connect(address).expect("Failed to connect");

        let noise = handshake_client!(SECRET, PARAMS, &mut stream).unwrap();

        Self {
            transport: Transport::new(stream, noise),
        }
    }

    pub fn authenticate(&mut self, username: &str, password: &str) {
        let packet = AuthenticationPacket::build((username.into(), password.into()));
        self.transport.send(&packet.to_bytes()).unwrap();
    }

    // hog the main thread
    pub fn run(&mut self) {
        let mut buffer = [0; 4096];
        loop {
            match self.transport.recv(&mut buffer) {
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
