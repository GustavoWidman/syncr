pub mod sanity;
pub mod size;

use super::{DynamicPacket, PacketBase, StaticPacket};

pub use sanity::SanityPacket;
pub use size::SizePacket;

#[derive(Debug)]
pub enum DynamicPackets {
    Sanity(SanityPacket),
}

#[derive(Debug)]
pub enum StaticPackets {
    Size(SizePacket),
}

#[derive(Debug)]
pub enum Packets {
    Sanity(SanityPacket),
    Size(SizePacket),
}
