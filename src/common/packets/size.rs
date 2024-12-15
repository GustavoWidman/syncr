use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::common::packet::PacketBase;

use super::super::Packet;
#[derive(Serialize, Deserialize, Debug, Default)]
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

impl Packet for SizePacket {}
