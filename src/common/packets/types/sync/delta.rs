use serde::{Deserialize, Serialize};

use super::{DynamicPacket, PacketBase};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SyncDeltaPacket {
    pub delta: Vec<u8>,
    pub new_file_size: usize,
}

// Pretty simple, contains the fat delta buffer and the expected file size
impl PacketBase for SyncDeltaPacket {
    const TYPE: &'static [u8; 4] = b"SDLT"; // sync delta (Sync DeLTa)
    type BuildParams = (Vec<u8>, usize);

    fn build(params: Self::BuildParams) -> Self {
        Self {
            delta: params.0,
            new_file_size: params.1,
        }
    }
}

impl DynamicPacket for SyncDeltaPacket {}
