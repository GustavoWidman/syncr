use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AuthenticationPacket {
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

#[derive(Serialize, Deserialize, Debug)]
struct DataPacket {
    data: Vec<u8>,
}

impl Packet for DataPacket {
    fn get_type(&self) -> String {
        "DataPacket".into()
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize")
    }
}
