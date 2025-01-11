use log::info;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::super::PacketBase;

pub async fn send_packet<S: AsyncWrite + Unpin, P: PacketBase>(
    packet: &P,
    stream: &mut S,
) -> Result<(), anyhow::Error> {
    let bytes = packet.to_bytes();

    info!("Writing packet: {:?}", bytes);

    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}
