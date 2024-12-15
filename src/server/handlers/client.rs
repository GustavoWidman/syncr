use futures::{FutureExt, TryFutureExt};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    task::{AbortHandle, JoinHandle},
};

use crate::common::{
    Packet, Packets, get_buffer_for_type, packetize, packets::size::SizePacket,
    stream::SecureStream,
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
            let header_buffer = Client::read_header(&mut stream).await?;
            let packet = match get_buffer_for_type(&header_buffer, &last_size_packet) {
                Ok(mut buffer) => {
                    Client::read_packet(&mut stream, &mut buffer).await?;

                    let packet = packetize(&header_buffer, buffer)?;

                    match packet {
                        Packets::Size(size_packet) => {
                            println!(
                                "Received size packet, ready for a packet of size {:?}",
                                size_packet.packet_size
                            );
                            last_size_packet = Some(size_packet);
                            continue;
                        }
                        other_packet => {
                            last_size_packet = None;
                            other_packet
                        }
                    }
                }
                Err(e) => {
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
        }
    }

    pub async fn read_packet(
        stream: &mut SecureStream,
        packet_buffer: &mut Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        let expected_size = packet_buffer.capacity();

        let mut bytes_read = 0;
        while bytes_read < expected_size {
            let n = stream.reader.read(&mut packet_buffer[bytes_read..]).await?;
            if n == 0 {
                return Err(anyhow::anyhow!("Unexpected EOF"));
            }
            bytes_read += n;
        }
        Ok(())
    }

    pub async fn read_header(stream: &mut SecureStream) -> io::Result<([u8; 4])> {
        let mut header_buffer = [0u8; 4];

        let n = stream.reader.read_exact(&mut header_buffer).await?;

        Ok(header_buffer)
    }
}
