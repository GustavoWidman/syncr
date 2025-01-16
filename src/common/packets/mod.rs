mod base;
mod dynamic;
mod mmap;
mod r#static;
mod utils;

pub mod types;

use utils::packet_buffer_mapper;

pub use base::PacketBase;
pub use dynamic::DynamicPacket;
pub use r#static::StaticPacket;
pub use utils::packetize;

pub use types::{DynamicPackets, Packets, SanityPacket, SizePacket, StaticPackets};

packet_buffer_mapper!(
    SizePacket
    ;
    SanityPacket
);
