use std::{
    io::{BufWriter, Write},
    sync::Arc,
};

use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::{Mutex, mpsc},
};

use super::{
    Packets,
    packets::{self, sanity::SanityPacket, size::SizePacket},
};

pub trait PacketBase:
    serde::Serialize + serde::de::DeserializeOwned + Send + Sync + Default + 'static
{
    type BuildParams;
    const TYPE: &'static [u8; 4];

    fn build(params: Self::BuildParams) -> Self;
    fn get_type(&self) -> &[u8; 4] {
        Self::TYPE
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

    // fn arc(self) -> Arc<Self> {
    //     Arc::new(self)
    // }
}

pub trait Packet: PacketBase {
    async fn write(&self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<(), anyhow::Error> {
        write_packet(self, stream).await
    }

    fn make_buffer() -> Result<Vec<u8>, anyhow::Error> {
        let mut serialized_size = bincode::serialized_size(&Self::default())? as usize;
        let mut buffer = Vec::with_capacity(serialized_size);
        buffer.resize(serialized_size, 0);

        Ok(buffer)
    }
}

pub trait DynamicPacket: PacketBase {
    async fn write(&self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<(), anyhow::Error> {
        let size_packet = self.get_size();

        size_packet.write(stream).await?;

        println!("Writing packet");

        write_packet(self, stream).await
    }

    fn get_size(&self) -> SizePacket {
        SizePacket {
            packet_size: bincode::serialized_size(&self).unwrap() as u64,
        }
    }
    fn make_buffer(size: &SizePacket) -> Result<Vec<u8>, anyhow::Error> {
        let capacity = size.packet_size as usize;
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, 0);
        Ok(buffer)
    }
}

pub async fn write_packet(
    self_: &(impl PacketBase),
    stream: &mut (impl AsyncWrite + Unpin),
) -> Result<(), anyhow::Error> {
    let bytes = self_.to_bytes();
    println!("Writing packet: {:?}", bytes);
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}

macro_rules! packet_buffer_mapper {
    ($($static_packet:ident),* ; $($dynamic_packet:ident),*) => {
        pub fn get_buffer_for_type(packet_type: &[u8; 4], size: &Option<SizePacket>) -> Result<Vec<u8>, anyhow::Error> {
            match (packet_type, size) {
                // Static packets (ignore size parameter)
                $(
                    ($static_packet::TYPE, _) => $static_packet::make_buffer(),
                )*
                // Dynamic packets (require size parameter)
                $(
                    ($dynamic_packet::TYPE, Some(size)) => <$dynamic_packet as crate::common::packet::DynamicPacket>::make_buffer(&size),
                    ($dynamic_packet::TYPE, None) => panic!("Dynamic packet requires size parameter"),
                )*
                _ => panic!("Unknown packet type: {:?}", packet_type)
            }
        }
    };
}

pub(super) use packet_buffer_mapper;

pub fn packetize(packet_type: &[u8; 4], packet_buf: Vec<u8>) -> Result<Packets, anyhow::Error> {
    let packet_type = std::str::from_utf8(packet_type)?;
    let packet_type = packet_type.to_uppercase();

    match packet_type.as_str() {
        "NONC" => Ok(Packets::Nonce(packets::nonce::NoncePacket::from_bytes(
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
