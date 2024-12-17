use futures::{FutureExt, TryFutureExt};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    task::{AbortHandle, JoinHandle},
};

use crate::common::{
    packetize,
    packets::{Packets, SizePacket, StaticPackets, get_buffer_for_type},
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
            println!("Header: {:?}", header_buffer);
            let packet = match get_buffer_for_type(&header_buffer, &last_size_packet) {
                Ok(mut buffer) => {
                    Client::read_packet(&mut stream, &mut buffer).await?;

                    println!("Packet: {:?}", buffer);

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
                            if last_size_packet.is_some() {
                                last_size_packet = None;
                            }

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
            let n = stream.read(&mut packet_buffer[bytes_read..]).await?;
            if n == 0 {
                return Err(anyhow::anyhow!("Unexpected EOF"));
            }
            bytes_read += n;
        }
        Ok(())
    }

    pub async fn read_header(stream: &mut SecureStream) -> io::Result<([u8; 4])> {
        let expected_size = 4;
        let mut buf = [0u8; 4];
        buf.fill(0);

        let mut bytes_read = 0;
        while bytes_read < expected_size {
            let n = stream.read(&mut buf[bytes_read..]).await?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF",
                ));
            }
            bytes_read += n;
        }

        Ok(buf)
    }
}
