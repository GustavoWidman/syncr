pub mod sanity;
pub mod size;
pub mod tiebreak;

use super::{DynamicPacket, PacketBase, StaticPacket};

pub use sanity::SanityPacket;
pub use size::SizePacket;
pub use tiebreak::TieBreakPacket;

#[derive(Debug)]
pub enum DynamicPackets {
    Sanity(SanityPacket),
}

#[derive(Debug)]
pub enum StaticPackets {
    TieBreak(TieBreakPacket),
    Size(SizePacket),
}

#[derive(Debug)]
pub enum Packets {
    Sanity(SanityPacket),
    TieBreak(TieBreakPacket),
    Size(SizePacket),
}
