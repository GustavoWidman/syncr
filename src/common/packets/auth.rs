use serde::{Deserialize, Serialize};

use super::super::Packet;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationPacket {
    username: String,
    password: String,
}

impl Packet for AuthenticationPacket {
    fn get_type(&self) -> String {
        "AuthenticationPacket".into()
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize")
    }
}
