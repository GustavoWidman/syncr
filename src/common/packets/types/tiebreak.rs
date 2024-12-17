use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};

use super::{PacketBase, StaticPacket};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TieBreakPacket {
    pub random: u64,
}

impl PacketBase for TieBreakPacket {
    const TYPE: &'static [u8; 4] = b"TYBR"; // haha, get it? ty-e br-eak
    type BuildParams = ();

    fn build(params: Self::BuildParams) -> Self {
        let mut rng = OsRng;
        let mut random = [0u8; 8];
        rng.fill_bytes(&mut random);
        TieBreakPacket {
            random: u64::from_le_bytes(random),
        }
    }
}

impl StaticPacket for TieBreakPacket {}
