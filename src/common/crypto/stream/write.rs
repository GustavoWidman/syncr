use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Buf;
use futures::{ready, Future, TryFutureExt};
use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use super::nonce;
use super::SecureStream;

impl AsyncWrite for SecureStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Custom future to perform async writes as we need to await the `send_buffer` operation
        let future = async {
            match self.send_buffer(buf).await {
                Ok(_) => Ok(buf.len()), // On successful encryption and send, return the length of data sent
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
            }
        };

        // Pin and poll the future
        let mut pin = Box::pin(future);
        pin.as_mut().poll(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // No extra flushing is needed for our scenario, just delegate to inner stream's flush
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Properly close the connection by shutting down the stream

        let future = async {
            let mut stream = self.stream.lock().await;
            stream
                .shutdown()
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        };

        let mut pin = Box::pin(future);
        pin.as_mut().poll(cx)
    }
}
