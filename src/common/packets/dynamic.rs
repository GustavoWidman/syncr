use super::{SizePacket, StaticPacket};
use log::info;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub trait DynamicPacket:
    super::PacketBase + serde::de::DeserializeOwned + serde::Serialize
{
    async fn write<S: AsyncWrite + Unpin>(&self, stream: &mut S) -> Result<(), anyhow::Error> {
        let size_packet = self.get_size();

        size_packet.write(stream).await?;

        let bytes = self.to_bytes();

        info!("Writing packet: {:?}", bytes);

        stream.write_all(&bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    fn get_size(&self) -> SizePacket {
        SizePacket {
            packet_size: bincode::serialized_size(&self).unwrap() as u64,
        }
    }
    fn make_buffer(size: &SizePacket) -> Result<Vec<u8>, anyhow::Error> {
        let capacity = size.packet_size as usize;
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, 0);
        Ok(buffer)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(&bytes).expect("Failed to deserialize")
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        std::io::Write::write_all(&mut buf, self.get_type()).unwrap();
        bincode::serialize_into(&mut buf, self).expect("Failed to serialize");
        buf
    }
}
