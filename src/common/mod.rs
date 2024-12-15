mod packet;
pub mod packets;
pub mod stream;
pub mod sync;

use packets::{nonce::NoncePacket, sanity::SanityPacket, size::SizePacket};

#[derive(Debug)]
pub enum Packets {
    // Authentication(packets::auth::AuthenticationPacket),
    Nonce(NoncePacket),
    Size(SizePacket),
    Sanity(SanityPacket),
    // Data(DataPacket),
}

use packet::packet_buffer_mapper;

packet_buffer_mapper!(
    NoncePacket,
    SizePacket
    ;
    SanityPacket
);

#[derive(Debug)]
pub enum DynamicPackets {
    Sanity(packets::sanity::SanityPacket),
}

use std::collections::HashMap;
use std::iter::Map;
use std::sync::Arc;

// exports
pub(crate) use packet::DynamicPacket;
pub(crate) use packet::Packet;
pub(crate) use packet::PacketBase;
pub(crate) use packet::packetize;
pub(self) use packet::write_packet;
