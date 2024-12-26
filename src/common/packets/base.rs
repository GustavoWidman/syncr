use tokio::io::{AsyncWrite, AsyncWriteExt};

pub trait PacketBase:
    serde::Serialize + serde::de::DeserializeOwned + Send + Sync + Default + Clone + 'static
{
    type BuildParams;
    const TYPE: &'static [u8; 4];

    fn build(params: Self::BuildParams) -> Self;
    fn get_type(&self) -> &[u8; 4] {
        Self::TYPE
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
