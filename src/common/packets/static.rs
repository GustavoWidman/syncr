use log::info;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub trait StaticPacket:
    super::PacketBase + std::default::Default + serde::de::DeserializeOwned + serde::Serialize
{
    async fn write<S: AsyncWrite + Unpin>(&self, stream: &mut S) -> Result<(), anyhow::Error> {
        let bytes = self.to_bytes();

        info!("Writing packet: {:?}", bytes);

        stream.write_all(&bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    fn make_buffer() -> Result<Vec<u8>, anyhow::Error> {
        let serialized_size = bincode::serialized_size(&Self::default())? as usize;
        let mut buffer = Vec::with_capacity(serialized_size);
        buffer.resize(serialized_size, 0);

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
