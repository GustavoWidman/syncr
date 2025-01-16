pub mod sanity;
pub mod size;
pub mod sync;

use super::{DynamicPacket, PacketBase, StaticPacket};

pub use sanity::SanityPacket;
pub use size::SizePacket;
pub use sync::ack::SyncAcknowledgePacket;
pub use sync::delta::SyncDeltaPacket;
pub use sync::init::SyncInitPacket;

#[derive(Debug)]
pub enum DynamicPackets {
    Sanity(SanityPacket),
    SyncAck(SyncAcknowledgePacket),
    SyncDelta(SyncDeltaPacket),
}

#[derive(Debug)]
pub enum StaticPackets {
    Size(SizePacket),
    SyncInit(SyncInitPacket),
}

#[derive(Debug)]
pub enum Packets {
    Sanity(SanityPacket),
    Size(SizePacket),
    SyncInit(SyncInitPacket),
    SyncAck(SyncAcknowledgePacket),
    SyncDelta(SyncDeltaPacket),
}
