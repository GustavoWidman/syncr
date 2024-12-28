use serde::{Deserialize, Serialize};

use super::{PacketBase, StaticPacket};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SizePacket {
    pub packet_size: u64,
}

// This is a packet that is sent before dynamically sized packets
// It is used to determine the size of the packet
impl PacketBase for SizePacket {
    const TYPE: &'static [u8; 4] = b"SIZE";
    type BuildParams = u64;

    fn build(params: Self::BuildParams) -> Self {
        Self {
            packet_size: params,
        }
    }
}

impl StaticPacket for SizePacket {}
