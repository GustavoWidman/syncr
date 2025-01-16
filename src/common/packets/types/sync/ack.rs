use serde::{Deserialize, Serialize};

use super::{DynamicPacket, PacketBase};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SyncAcknowledgePacket {
    pub ack: bool,
    pub data: Option<AckData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AckData {
    pub signature: Vec<u8>,
    pub block_size: u32,
}

// This packet is sent to acknowledge if a SYNC transaction should begin occuring
// This means, that the hash sent from the INIT packet told us that we had a different
// file and that we should be syncing :3
impl PacketBase for SyncAcknowledgePacket {
    const TYPE: &'static [u8; 4] = b"SACK"; // get it? sync ack? sack haha
    type BuildParams = (bool, Option<AckData>);

    fn build(params: Self::BuildParams) -> Self {
        Self {
            ack: params.0,
            data: params.1,
        }
    }
}

impl DynamicPacket for SyncAcknowledgePacket {}
