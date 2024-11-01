use std::io::Write;

use serde::{Deserialize, Serialize};

use super::super::Packet;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationPacket {
    username: String,
    password: String,
}

impl Packet for AuthenticationPacket {
    type BuildParams = (String, String);

    fn build(params: Self::BuildParams) -> Self {
        AuthenticationPacket {
            username: params.0,
            password: params.1,
        }
    }

    fn get_type(&self) -> &[u8; 4] {
        b"AUTH"
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize")
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.write_all(b"AUTH").unwrap();
        serde_json::to_writer(&mut buf, self).expect("Failed to serialize");
        buf
    }
}
