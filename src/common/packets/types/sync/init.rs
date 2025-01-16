use serde::{Deserialize, Serialize};

use super::{PacketBase, StaticPacket};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncInitPacket {
    pub hash: blake3::Hash,
    pub syncr_id: String,
    pub known_name: String,
}

impl Default for SyncInitPacket {
    fn default() -> Self {
        Self {
            hash: blake3::Hasher::new().finalize(),
            syncr_id: Default::default(),
            known_name: Default::default(),
        }
    }
}

// This packet is sent to initialize a SYNC transaction
impl PacketBase for SyncInitPacket {
    const TYPE: &'static [u8; 4] = b"INIT";
    type BuildParams = (blake3::Hash, String, String);

    fn build(params: Self::BuildParams) -> Self {
        Self {
            hash: params.0,
            syncr_id: params.1,
            known_name: params.2,
        }
    }
}

impl StaticPacket for SyncInitPacket {}
