use anyhow::anyhow;
use rand::{rngs::OsRng, RngCore};
use ring::{
    aead::NONCE_LEN,
    pbkdf2::{self, PBKDF2_HMAC_SHA256},
    rand::{SecureRandom, SystemRandom},
};
use std::num::NonZeroU32;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::common::{packets::salt::SaltPacket, Packet};

// reference to https://github.com/david-wiles/rust-ring-aead
// Create a new random initialization vector, or counter, to use in a NonceSequence
pub fn new_iv() -> Result<[u8; NONCE_LEN], anyhow::Error> {
    let mut nonce_buf = [0u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce_buf)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(nonce_buf)
}

// reference to https://github.com/david-wiles/rust-ring-aead
pub fn derive_key_from_pass(
    pass: String,
    salt: &[u8],
    iters: u32,
) -> Result<[u8; 32], anyhow::Error> {
    let mut key = [0u8; 32];
    let iterations = NonZeroU32::new(iters).ok_or(anyhow!("Invalid number of iterations"))?;

    pbkdf2::derive(
        PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        &pass.as_bytes(),
        &mut key,
    );

    Ok(key)
}

pub fn new_salt() -> SaltPacket {
    let mut rng = OsRng;
    let mut salt = [0u8; 12];
    rng.fill_bytes(&mut salt);
    SaltPacket::build(salt)
}
