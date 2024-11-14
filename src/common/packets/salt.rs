use std::io::Write;

use rand::{rngs::OsRng, RngCore};
use ring::rand::SystemRandom;
use serde::{Deserialize, Serialize};

use super::super::Packet;

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

    fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize")
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.write_all(b"SALT").unwrap();
        serde_json::to_writer(&mut buf, self).expect("Failed to serialize");
        buf
    }
}
