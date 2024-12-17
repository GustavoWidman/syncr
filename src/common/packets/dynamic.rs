use super::{SizePacket, StaticPacket};
use tokio::io::AsyncWrite;

pub trait DynamicPacket: super::PacketBase {
    async fn write<S: AsyncWrite + Unpin>(&self, stream: &mut S) -> Result<(), anyhow::Error> {
        let size_packet = self.get_size();

        size_packet.write(stream).await?;

        super::send_packet(self, stream).await
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
}
