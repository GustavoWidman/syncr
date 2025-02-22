use serde::{Deserialize, Serialize};

use super::{DynamicPacket, PacketBase};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SanityPacket {
    pub message: Vec<u8>,
}

// This is a packet that is sent before dynamically sized packets
// It is used to determine the size of the packet
impl PacketBase for SanityPacket {
    const TYPE: &'static [u8; 4] = b"SNTY";
    type BuildParams = Vec<u8>;

    fn build(params: Self::BuildParams) -> Self {
        Self { message: params }
    }
}

impl DynamicPacket for SanityPacket {}
