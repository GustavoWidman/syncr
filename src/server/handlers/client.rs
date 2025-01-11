use tokio::{
    io::{self, AsyncReadExt, Interest},
    task::AbortHandle,
};

use crate::common::{
    packetize,
    packets::{Packets, SizePacket, get_buffer_for_type},
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
            let ready = stream
                .ready(Interest::READABLE | Interest::WRITABLE)
                .await?;
            if !ready.is_readable() || !ready.is_writable() {
                continue;
            }

            let packet = Client::extract_packet(&mut stream, &mut last_size_packet).await?;

            match packet {
                // dont handle size packets
                Packets::Size(_) => {
                    continue;
                }
                _ => Client::handle_packet(packet, &mut stream)?,
            }
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

    pub async fn read_header(stream: &mut SecureStream) -> io::Result<[u8; 4]> {
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

    pub async fn extract_packet(
        mut stream: &mut SecureStream,
        last_size_packet: &mut Option<SizePacket>,
    ) -> Result<Packets, anyhow::Error> {
        let header_buffer = Client::read_header(&mut stream).await?;

        // info!("Header: {:?}", header_buffer);
        let packet = match get_buffer_for_type(&header_buffer, last_size_packet) {
            Ok(mut buffer) => {
                Client::read_packet(&mut stream, &mut buffer).await?;

                // info!("Packet: {:?}", buffer);

                let packet = packetize(&header_buffer, buffer)?;

                match packet {
                    Packets::Size(size_packet) => {
                        // info!(
                        //     "Received size packet, ready for a packet of size {:?}",
                        //     size_packet.packet_size
                        // );
                        *last_size_packet = Some(size_packet.clone());
                        return Ok(Packets::Size(size_packet));
                    }
                    other_packet => {
                        if last_size_packet.is_some() {
                            *last_size_packet = None;
                        }

                        other_packet
                    }
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(packet)
    }

    pub fn handle_packet(packet: Packets, stream: &mut SecureStream) -> Result<(), anyhow::Error> {
        todo!()
    }
}
