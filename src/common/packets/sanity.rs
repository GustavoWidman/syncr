use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{super::write_dynsized_packet, super::Packet, size::SizePacket};
#[derive(Serialize, Deserialize, Debug)]
pub struct SanityPacket {
    pub message: Vec<u8>,
}

// This is a packet that is sent before dynamically sized packets
// It is used to determine the size of the packet
impl Packet for SanityPacket {
    type BuildParams = Vec<u8>;

    fn build(params: Self::BuildParams) -> Self {
        Self { message: params }
    }

    fn get_type(&self) -> &[u8; 4] {
        b"SNTY"
    }

    fn get_size(&self) -> Option<SizePacket> {
        Some(SizePacket {
            packet_size: bincode::serialized_size(&self).unwrap() as u64,
        })
    }

    fn make_buffer(size: &Option<SizePacket>) -> Result<Vec<u8>, anyhow::Error> {
        Ok(Vec::with_capacity(
            size.as_ref()
                .ok_or(anyhow::anyhow!(
                    "Missing size packet for dynamically sized packet \"SNTY\""
                ))?
                .packet_size as usize,
        ))
    }

    async fn write(
        self: std::sync::Arc<Self>,
        stream: std::sync::Arc<
            tokio::sync::Mutex<(impl tokio::io::AsyncWriteExt + Unpin + Send + 'static)>,
        >,
    ) -> Result<usize, anyhow::Error> {
        write_dynsized_packet(self, stream).await
    }
}
