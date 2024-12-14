use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use super::super::Packet;
#[derive(Serialize, Deserialize, Debug)]
pub struct SizePacket {
    pub packet_size: u64,
}

// This is a packet that is sent before dynamically sized packets
// It is used to determine the size of the packet
impl Packet for SizePacket {
    type BuildParams = u64;

    fn build(params: Self::BuildParams) -> Self {
        Self {
            packet_size: params,
        }
    }

    fn get_type(&self) -> &[u8; 4] {
        b"SIZE"
    }

    fn make_buffer(_: &Option<SizePacket>) -> Result<Vec<u8>, anyhow::Error> {
        let mut serialized_size = bincode::serialized_size(&SizePacket::build(0)).unwrap() as usize;

        Ok(Vec::with_capacity(serialized_size))
    }
}
