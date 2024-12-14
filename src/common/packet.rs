use std::{
    io::{BufWriter, Write},
    sync::Arc,
};

use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::{mpsc, Mutex},
};

use super::{
    packets::{self, size::SizePacket},
    Packets,
};

pub trait Packet: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static {
    type BuildParams;
    fn build(params: Self::BuildParams) -> Self;
    fn get_type(&self) -> &[u8; 4];
    fn make_buffer(size: &Option<SizePacket>) -> Result<Vec<u8>, anyhow::Error>;
    fn get_size(&self) -> Option<SizePacket> {
        None
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(&bytes).expect("Failed to deserialize")
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        std::io::Write::write_all(&mut buf, self.get_type()).unwrap();
        bincode::serialize_into(&mut buf, self).expect("Failed to serialize");
        buf
    }

    fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    async fn write(
        self: Arc<Self>,
        stream: Arc<Mutex<(impl AsyncWriteExt + Unpin + Send + 'static)>>,
    ) -> Result<usize, anyhow::Error> {
        write_packet(self, stream).await
    }
}

pub async fn write_packet(
    self_: Arc<impl Packet>,
    stream: Arc<Mutex<(impl AsyncWriteExt + Unpin + Send + 'static)>>,
) -> Result<usize, anyhow::Error> {
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(4096);

    // Spawn a blocking task to handle serialization
    tokio::task::spawn_blocking(move || {
        let mut buffer = Vec::new();
        let mut writer = BufWriter::new(&mut buffer);

        let self_clone = self_.clone();
        writer.write_all(self_clone.get_type()).unwrap();
        bincode::serialize_into(&mut writer, &*self_clone).unwrap();
        writer.flush().unwrap();
        drop(writer); // Drop the writer to release the borrow on buffer

        tx.blocking_send(buffer).unwrap();
    });

    // Read from channel and write to async stream
    let mut total_bytes = 0;
    let mut stream = stream.lock().await;
    while let Some(buffer) = rx.recv().await {
        stream.write_all(&buffer).await?;
        total_bytes += buffer.len();
    }

    Ok(total_bytes)
}

pub async fn write_dynsized_packet(
    self_: Arc<impl Packet>,
    stream: Arc<Mutex<(impl AsyncWriteExt + Unpin + Send + 'static)>>,
) -> Result<usize, anyhow::Error> {
    let size_packet = self_
        .get_size()
        .ok_or(anyhow::anyhow!(
            "Missing size packet for dynamically sized packet \"{}\"",
            std::str::from_utf8(self_.get_type())?
        ))?
        .arc();

    size_packet.clone().write(stream.clone()).await?;

    write_packet(self_, stream).await
}

// yes i know this function is stupid... i dont know how else to do this...
// todo: think of a better way to do this
pub fn make_packet_buffer(
    packet_type: &[u8; 4],
    size: &Option<SizePacket>,
) -> Result<Vec<u8>, anyhow::Error> {
    let packet_type = std::str::from_utf8(packet_type)?;

    match packet_type.to_uppercase().as_str() {
        "SALT" => Ok(packets::salt::SaltPacket::make_buffer(size)?),
        "SIZE" => Ok(packets::size::SizePacket::make_buffer(size)?),
        "SNTY" => Ok(packets::sanity::SanityPacket::make_buffer(size)?),
        _ => Err(anyhow::anyhow!("Invalid packet type")),
    }
}

pub fn packetize(packet_type: &[u8; 4], packet_buf: Vec<u8>) -> Result<Packets, anyhow::Error> {
    let packet_type = std::str::from_utf8(packet_type)?;
    let packet_type = packet_type.to_uppercase();

    match packet_type.as_str() {
        "SALT" => Ok(Packets::Salt(packets::salt::SaltPacket::from_bytes(
            &packet_buf,
        ))),
        "SIZE" => Ok(Packets::Size(packets::size::SizePacket::from_bytes(
            &packet_buf,
        ))),
        "SNTY" => Ok(Packets::Sanity(packets::sanity::SanityPacket::from_bytes(
            &packet_buf,
        ))),
        _ => Err(anyhow::anyhow!("Invalid packet type")),
    }
}
