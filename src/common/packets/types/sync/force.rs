use std::ops::Deref;

use memmap2::Mmap;
use serde::{Deserialize, Serialize};

use crate::common::packets::mmap::MmapPacket;

use super::{PacketBase, StaticPacket};

#[derive(Debug)]
pub struct SyncForcePacket {
    pub mmap: Mmap,

    pub inner: SyncForcePacketStatic,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SyncForcePacketStatic {
    pub syncr_id: String,
    pub known_name: String,
}

impl PacketBase for SyncForcePacketStatic {
    const TYPE: &'static [u8; 4] = b"FRCE";
    type BuildParams = (String, String);

    fn build(params: Self::BuildParams) -> Self {
        Self {
            syncr_id: params.0,
            known_name: params.1,
        }
    }
}
impl StaticPacket for SyncForcePacketStatic {}

// this packet is sent in desperation to sync a file
// that does not want to sync with normal delta
// diff. basically we send the whole file over lol
impl PacketBase for SyncForcePacket {
    const TYPE: &'static [u8; 4] = b"MMAP";
    type BuildParams = (Mmap, String, String);

    fn build(params: Self::BuildParams) -> Self {
        Self {
            mmap: params.0,
            inner: SyncForcePacketStatic::build((params.1, params.2)),
        }
    }
}

impl MmapPacket for SyncForcePacket {
    type MmaplessPacket = SyncForcePacketStatic;

    fn get_mmap(&self) -> &Mmap {
        &self.mmap
    }
    fn get_mmapless(&self) -> &Self::MmaplessPacket {
        &self.inner
    }

    async fn deserialize<S: tokio::io::AsyncRead + Unpin>(reader: &mut S) -> anyhow::Result<Self> {
        let (mmap, inner) = Self::deserialize_inner(reader).await?;

        Ok(Self { mmap, inner })
    }
}

impl Deref for SyncForcePacket {
    type Target = SyncForcePacketStatic;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
