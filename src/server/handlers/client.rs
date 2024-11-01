use std::thread::JoinHandle;

use crate::common::{
    find_packet_type, noise::Transport, packets::auth::AuthenticationPacket, Packet,
};

pub struct Client {
    handle: JoinHandle<()>,
}

impl Client {
    pub fn new(handle: JoinHandle<()>) -> Self {
        Client { handle }
    }

    pub fn handle(mut transport: Transport) {
        let mut buffer = [0; 4096];
        loop {
            match transport.recv(&mut buffer) {
                Ok(0) => break,
                Ok(1..4) => continue,
                Ok(n) => {
                    let (packet_type, bytes) = find_packet_type(&buffer[..n]).unwrap();

                    println!("Received packet: {:?}", packet_type);

                    let packet = match packet_type {
                        "AUTH" => AuthenticationPacket::from_bytes(bytes),
                        _ => panic!("Invalid packet type"),
                    };

                    println!("Received packet: {:?}", packet);
                }
                Err(e) => {
                    eprintln!("Failed to read from stream: {:?}", e);
                    break;
                }
            }
        }

        println!("Client disconnected");
    }
}
