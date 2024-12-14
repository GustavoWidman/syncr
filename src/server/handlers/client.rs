use futures::{FutureExt, TryFutureExt};
use tokio::{
    io::AsyncReadExt,
    task::{AbortHandle, JoinHandle},
};

use crate::common::{
    crypto::SecureStream, make_packet_buffer, packetize, packets::size::SizePacket, Packet, Packets,
};

pub struct Client {
    handle: AbortHandle,
}

impl Client {
    pub fn new(handle: AbortHandle) -> Self {
        Client { handle }
    }

    pub async fn handle(mut stream: SecureStream) -> Result<(), anyhow::Error> {
        let mut last_size_packet: Option<SizePacket> = None;
        loop {
            let mut header_buffer = [0; 4];
            match stream.read_exact(&mut header_buffer).await {
                Ok(_) => {
                    let packet = match make_packet_buffer(&header_buffer, &last_size_packet) {
                        Ok(mut buffer) => {
                            Client::read_packet(&mut stream, &mut buffer).await?;
                            let packet = packetize(&header_buffer, buffer)?;

                            // prepare for a dynsized packet
                            if let Packets::Size(packet) = packet {
                                last_size_packet = Some(packet);
                                continue;
                            } else {
                                last_size_packet = None;
                            }

                            packet
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            return Err(e.into());
                        }
                    };

                    println!("Packet: {:?}", packet);
                    match packet {
                        Packets::Sanity(packet) => {
                            println!(
                                "Received sanity packet. Message: {:?}",
                                std::str::from_utf8(&packet.message)?
                            );
                        }
                        _ => {}
                    };

                    continue;
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    return Err(e.into());
                }
            }
        }
    }

    pub async fn read_packet(
        stream: &mut SecureStream,
        packet_buffer: &mut Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        stream.read_buf(packet_buffer).await?;
        Ok(())
    }
}
