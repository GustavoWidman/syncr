pub mod macros;
mod send;

pub(crate) use macros::packet_buffer_mapper;
pub use send::send_packet;

use super::{
    Packets,
    base::PacketBase,
    types::{SanityPacket, SizePacket},
};

pub fn packetize(packet_type: &[u8; 4], packet_buf: Vec<u8>) -> Result<Packets, anyhow::Error> {
    let packet_type = std::str::from_utf8(packet_type)?;
    let packet_type = packet_type.to_uppercase();

    match packet_type.as_str() {
        "SIZE" => Ok(Packets::Size(SizePacket::from_bytes(&packet_buf))),
        "SNTY" => Ok(Packets::Sanity(SanityPacket::from_bytes(&packet_buf))),
        _ => Err(anyhow::anyhow!("Invalid packet type")),
    }
}
