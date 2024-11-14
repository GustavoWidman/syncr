use futures::{FutureExt, TryFutureExt};
use tokio::{
    io::AsyncReadExt,
    task::{AbortHandle, JoinHandle},
};

use crate::common::{crypto::SecureStream, find_packet_type, Packet};

pub struct Client {
    handle: AbortHandle,
}

impl Client {
    pub fn new(handle: AbortHandle) -> Self {
        Client { handle }
    }

    pub async fn handle(mut stream: SecureStream) -> Result<(), anyhow::Error> {
        let mut buffer = [0; 4096];
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => return Ok(()),
                Ok(1..4) => continue,
                Ok(n) => {
                    let (packet_type, bytes) = find_packet_type(&buffer[..n]).unwrap();

                    println!("Received packet type: {:?}", packet_type);
                    println!("Received packet: {:?}", std::str::from_utf8(&bytes));
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}
