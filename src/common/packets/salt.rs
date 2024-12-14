use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{super::Packet, size::SizePacket};
#[derive(Serialize, Deserialize, Debug)]
pub struct SaltPacket {
    pub salt: [u8; 12],
    pub random: u64,
}

impl Packet for SaltPacket {
    type BuildParams = ([u8; 12]);

    fn build(params: Self::BuildParams) -> Self {
        let mut rng = OsRng;
        let mut random = [0u8; 8];
        rng.fill_bytes(&mut random);
        SaltPacket {
            salt: params,
            random: u64::from_le_bytes(random),
        }
    }

    fn get_type(&self) -> &[u8; 4] {
        b"SALT"
    }

    fn make_buffer(_: &Option<SizePacket>) -> Result<Vec<u8>, anyhow::Error> {
        let mut serialized_size =
            bincode::serialized_size(&SaltPacket::build(([0u8; 12]))).unwrap() as usize;

        Ok(Vec::with_capacity(serialized_size))
    }
}
