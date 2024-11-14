// reference to https://github.com/david-wiles/rust-ring-aead
use ring::aead::{Aad, BoundKey, OpeningKey, SealingKey, UnboundKey, AES_256_GCM, NONCE_LEN};

use super::nonce::InitializedNonceSequence;

pub struct AESEngine {
    key: [u8; 32],
    counter: InitializedNonceSequence,
}

use super::keys;

impl AESEngine {
    pub fn new(pass: String, salt: &[u8]) -> Result<Self, anyhow::Error> {
        let iv = keys::new_iv()?;
        let key = keys::derive_key_from_pass(pass, salt, 100_000)?;
        Ok(Self {
            key: key,
            counter: InitializedNonceSequence::new(iv),
        })
    }

    pub fn encrypt_bytes(&self, payload: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let nonce_bytes = self.counter.current();
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.key)
            .map_err(|e| anyhow::anyhow!("Failed to create AES-GCM unbound key: {}", e))?;

        let mut sealing_key = SealingKey::new(unbound_key, self.counter);
        let mut raw = payload.to_owned();
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut raw)
            .map_err(|e| anyhow::anyhow!("Failed to seal data with AES-GCM: {}", e))?;

        // Append the nonce to the beginning of the encrypted bytes
        let mut data = nonce_bytes.to_vec();
        data.append(&mut raw);

        Ok(data)
    }

    pub fn decrypt_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        // Split the incoming bytes at the nonce length
        let (nonce_bytes, bytes) = bytes.split_at(NONCE_LEN);
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.key)
            .map_err(|e| anyhow::anyhow!("Failed to create AES-GCM unbound key: {}", e))?;

        let mut opening_key = OpeningKey::new(
            unbound_key,
            InitializedNonceSequence::new(nonce_bytes.try_into()?),
        );

        let mut raw = bytes.to_owned();
        let plaintext = opening_key
            .open_in_place(Aad::empty(), &mut raw)
            .map_err(|e| anyhow::anyhow!("Failed to open data with AES-GCM: {}", e))?;
        Ok(plaintext.to_owned())
    }
}
