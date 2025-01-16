use std::{io::Write, ops::Deref};

use super::StaticPacket;
use memmap2::Mmap;
use tempfile::NamedTempFile;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};

pub trait MmapPacket: super::PacketBase + Sized + Deref<Target = Self::MmaplessPacket> {
    type MmaplessPacket: StaticPacket;

    // split the struct into a static packet and a mmap packet
    fn get_mmap(&self) -> &Mmap;
    fn get_mmapless(&self) -> &Self::MmaplessPacket;

    async fn write<S: AsyncWrite + Unpin>(&self, writer: &mut S) -> anyhow::Result<()> {
        let mut writer = BufWriter::new(writer);
        writer.write_all(b"MMAP").await?; // declare a MMAP-like packet

        writer.write_all(b"STAT").await?; // declare STATIC section
        let static_packet = self.get_mmapless();

        static_packet.write(&mut writer).await?;

        let mmap = self.get_mmap();

        writer.write_all(b"DATA").await?; // declare DATA section
        writer.write_all(&mmap.len().to_le_bytes()).await?;
        writer.write_all(mmap.as_ref()).await?;

        writer.write_all(b"DONE").await?; // finish

        writer.flush().await?;

        Ok(())
    }

    async fn deserialize<S: AsyncRead + Unpin>(reader: &mut S) -> anyhow::Result<Self>;
    async fn deserialize_inner<S: AsyncRead + Unpin>(
        reader: &mut S,
    ) -> anyhow::Result<(Mmap, Self::MmaplessPacket)> {
        // given that we've already read "MMAP" header from the stream, we should wait for a "STAT" header
        let buf = read_header(reader).await?;
        if buf != *b"STAT" {
            return Err(anyhow::anyhow!("Expected STAT header"));
        }

        // read the static packet into bincode
        let mut buf = Self::MmaplessPacket::make_buffer()?;
        read_packet(reader, &mut buf).await?;
        let static_packet = Self::MmaplessPacket::from_bytes(&buf);

        // wait for data header
        let buf = read_header(reader).await?;
        if buf != *b"DATA" {
            return Err(anyhow::anyhow!("Expected DATA header"));
        }

        // read the size of the mmap
        let mut buf = vec![0u8; 8];
        read_packet(reader, &mut buf).await?;
        let mmap_size = usize::from_le_bytes(
            buf.try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert mmap size to usize"))?,
        );

        // open a temporary file
        let mut temp_file = NamedTempFile::new()?;
        // write MMAP_SIZE bytes from the reader to the file, until we've read the entire mmap
        let mut temp_buf = vec![0u8; 4096];
        let mut bytes_read = 0;
        while bytes_read < mmap_size {
            let n = reader.read(&mut temp_buf).await?;
            if n == 0 {
                return Err(anyhow::anyhow!("Unexpected EOF"));
            }
            bytes_read += n;
            temp_file.write_all(&temp_buf[..n])?;
        }
        temp_file.flush()?;

        // read the data header
        let buf = read_header(reader).await?;
        if buf != *b"DONE" {
            return Err(anyhow::anyhow!("Expected DONE header"));
        }

        Ok((unsafe { Mmap::map(&temp_file)? }, static_packet))
    }
}

pub async fn read_packet<T: AsyncRead + Unpin>(
    stream: &mut T,
    packet_buffer: &mut Vec<u8>,
) -> Result<(), anyhow::Error> {
    let expected_size = packet_buffer.capacity();

    let mut bytes_read = 0;
    while bytes_read < expected_size {
        let n = stream.read(&mut packet_buffer[bytes_read..]).await?;
        if n == 0 {
            return Err(anyhow::anyhow!("Unexpected EOF"));
        }
        bytes_read += n;
    }
    Ok(())
}

pub async fn read_header<T: AsyncRead + Unpin>(stream: &mut T) -> std::io::Result<[u8; 4]> {
    let expected_size = 4;
    let mut buf = [0u8; 4];
    buf.fill(0);

    let mut bytes_read = 0;
    while bytes_read < expected_size {
        let n = stream.read(&mut buf[bytes_read..]).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Unexpected EOF",
            ));
        }
        bytes_read += n;
    }

    Ok(buf)
}
