use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::bail;
use fixedstr::zstr;
use rand::{RngCore, rngs::OsRng};
use snowstorm::{Keypair, NoiseStream, snow::HandshakeState};
use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, Interest, ReadBuf, Ready},
    net::TcpStream,
};

use super::packets::{PacketBase, StaticPacket, TieBreakPacket};

static HANDSHAKE_PARAM: &str = "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2b";
static TRANSPORT_PARAM: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2b";

pub struct SecureStream {
    inner: NoiseStream<TcpStream>,
}

impl SecureStream {
    async fn handshake(
        mut stream: TcpStream,
        password: &[u8],
    ) -> Result<NoiseStream<TcpStream>, anyhow::Error> {
        let mut rng = OsRng;
        let mut random = [0u8; 8];
        rng.fill_bytes(&mut random);
        let my_rng = u64::from_le_bytes(random);

        stream.write_all(&random).await?;
        stream.flush().await?;

        // packet collection
        let mut buf = [0u8; 8];
        let n = stream.read_exact(&mut buf).await?;
        let their_rng = u64::from_le_bytes(buf);

        match my_rng > their_rng {
            true => {
                // secure initiator using 448 curve, psk on step 2
                let keypair =
                    snowstorm::Builder::new(HANDSHAKE_PARAM.parse()?).generate_keypair()?;
                let initiator = snowstorm::Builder::new(HANDSHAKE_PARAM.parse()?)
                    .local_private_key(&keypair.private)
                    .psk(3, password)
                    .build_initiator()?;

                let mut handshake_stream = NoiseStream::handshake(stream, initiator).await?;

                let keypair =
                    snowstorm::Builder::new(TRANSPORT_PARAM.parse()?).generate_keypair()?;

                handshake_stream.write_all(&keypair.public).await?;
                handshake_stream.flush().await?;

                let mut public = [0u8; 32];
                handshake_stream.read_exact(&mut public).await?;

                let initiator = snowstorm::Builder::new(TRANSPORT_PARAM.parse()?)
                    .local_private_key(&keypair.private)
                    .remote_public_key(&public)
                    .build_initiator()?;

                Ok(NoiseStream::handshake(handshake_stream.into_inner(), initiator).await?)
            }
            false => {
                let keypair =
                    snowstorm::Builder::new(HANDSHAKE_PARAM.parse()?).generate_keypair()?;
                let responder = snowstorm::Builder::new(HANDSHAKE_PARAM.parse()?)
                    .local_private_key(&keypair.private)
                    .psk(3, password)
                    .build_responder()?;

                let mut handshake_stream = NoiseStream::handshake(stream, responder).await?;

                let keypair =
                    snowstorm::Builder::new(TRANSPORT_PARAM.parse()?).generate_keypair()?;

                handshake_stream.write_all(&keypair.public).await?;
                handshake_stream.flush().await?;
                let mut public = [0u8; 32];
                handshake_stream.read_exact(&mut public).await?;

                let responder = snowstorm::Builder::new(TRANSPORT_PARAM.parse()?)
                    .local_private_key(&keypair.private)
                    .remote_public_key(&public)
                    .build_responder()?;

                Ok(NoiseStream::handshake(handshake_stream.into_inner(), responder).await?)
            }
        }
    }

    pub async fn new(
        mut stream: TcpStream,
        password_str: &zstr<32>,
    ) -> Result<Self, anyhow::Error> {
        // todo replace with config or .env reading (maybe read from .syncr dir)
        let mut password = [0u8; 32];
        fill_buffer(&mut password, password_str);

        let mut encrypted_stream = Self::handshake(stream, &password).await?;

        Ok(Self {
            inner: encrypted_stream,
        })
    }

    pub async fn ready(&self, interest: Interest) -> io::Result<Ready> {
        self.inner.get_inner().ready(interest).await
    }
}

// re-expose traits of inner
impl Deref for SecureStream {
    type Target = NoiseStream<TcpStream>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SecureStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn fill_buffer(buf: &mut [u8], s: &zstr<32>) {
    // Zero the buffer first
    buf.fill(0);

    // Copy as much of the string as will fit
    let bytes = s.as_bytes();
    let len = buf.len().min(bytes.len());
    buf[..len].copy_from_slice(&bytes[..len]);
}
