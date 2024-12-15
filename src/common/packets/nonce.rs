use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};

use crate::common::packet::PacketBase;

use super::{super::Packet, size::SizePacket};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NoncePacket {
    pub nonce: [u8; 12],
    pub random: u64,
}

impl PacketBase for NoncePacket {
    const TYPE: &'static [u8; 4] = b"NONC";
    type BuildParams = [u8; 12];

    fn build(params: Self::BuildParams) -> Self {
        let mut rng = OsRng;
        let mut random = [0u8; 8];
        rng.fill_bytes(&mut random);
        NoncePacket {
            nonce: params,
            random: u64::from_le_bytes(random),
        }
    }
}

impl Packet for NoncePacket {}
