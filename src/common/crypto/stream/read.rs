use bytes::BytesMut;
use futures::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncRead, AsyncReadExt, ReadBuf};

use super::SecureStream;

pub struct SecureReader {
    buffer: BytesMut, // Buffer for decrypted data
    position: usize,  // Current position in the buffer
}

impl SecureReader {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(4096),
            position: 0,
        }
    }
}

impl AsyncRead for SecureStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Check if the buffer has unread decrypted data
        if self.reader.position >= self.reader.buffer.len() {
            // Reset buffer and fill it with new decrypted data
            // Custom future to enable async operations due to `await`
            let future = async {
                let mut read_buf = vec![0u8; 4096]; // Temporary buffer to receive raw data
                self.reader.position = 0; // Reset the buffer position

                match self.recv_buffer(&mut read_buf).await {
                    Ok(decrypted_size) => {
                        // Resize the buffer to the size of the decrypted data
                        self.reader.buffer.resize(decrypted_size, 0);
                        self.reader.buffer[..decrypted_size]
                            .copy_from_slice(&read_buf[..decrypted_size]);
                        Ok(())
                    }
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
                }
            };

            // Pin and poll the future
            let mut pin = Box::pin(future);
            if let Poll::Pending = pin.as_mut().poll(cx) {
                return Poll::Pending;
            }
        }

        // Copy data from the buffer to the provided buf
        let available_data = &self.reader.buffer[self.reader.position..];

        let len = std::cmp::min(available_data.len(), buf.remaining());
        buf.put_slice(&available_data[..len]);
        self.reader.position += len;

        Poll::Ready(Ok(()))
    }
}
