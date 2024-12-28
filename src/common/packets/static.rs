use tokio::io::AsyncWrite;

pub trait StaticPacket: super::PacketBase {
    async fn write<S: AsyncWrite + Unpin>(&self, stream: &mut S) -> Result<(), anyhow::Error> {
        super::send_packet(self, stream).await
    }

    fn make_buffer() -> Result<Vec<u8>, anyhow::Error> {
        let serialized_size = bincode::serialized_size(&Self::default())? as usize;
        let mut buffer = Vec::with_capacity(serialized_size);
        buffer.resize(serialized_size, 0);

        Ok(buffer)
    }
}
