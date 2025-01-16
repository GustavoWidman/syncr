use futures::task::noop_waker_ref;
use std::io::{Result, Write};
use std::task::{Context, Poll};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::pin;

pub struct SyncWriteWrapper<T> {
    inner: T,
}

impl<T> SyncWriteWrapper<T>
where
    T: AsyncWrite,
{
    pub fn new(writer: T) -> Self {
        SyncWriteWrapper { inner: writer }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Write for SyncWriteWrapper<T>
where
    T: AsyncWrite + Unpin,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let write_fut = self.inner.write(buf);
        pin!(write_fut);
        match write_fut
            .as_mut()
            .poll(&mut Context::from_waker(noop_waker_ref()))
        {
            Poll::Ready(Ok(n)) => Ok(n),
            Poll::Ready(Err(e)) => Err(e),
            Poll::Pending => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Write operation is pending",
            )),
        }
    }
    fn flush(&mut self) -> Result<()> {
        let flush_fut = self.inner.flush();
        pin!(flush_fut);
        match flush_fut
            .as_mut()
            .poll(&mut Context::from_waker(noop_waker_ref()))
        {
            Poll::Ready(Ok(())) => Ok(()),
            Poll::Ready(Err(e)) => Err(e),
            Poll::Pending => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Flush operation is pending",
            )),
        }
    }
}
