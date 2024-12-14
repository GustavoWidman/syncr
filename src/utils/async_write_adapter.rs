use std::io::Write;
use tokio::io::{AsyncWrite, AsyncWriteExt};

// First, create a wrapper that converts AsyncWrite to sync Write
pub struct AsyncWriteAdapter<W: AsyncWrite + Unpin> {
    pub writer: W,
    pub rt: tokio::runtime::Handle,
}

impl<W: AsyncWrite + Unpin> Write for AsyncWriteAdapter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.rt.block_on(self.writer.write(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.rt.block_on(self.writer.flush())
    }
}
