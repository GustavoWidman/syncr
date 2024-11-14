// reference to https://github.com/david-wiles/rust-ring-aead
use ring::aead::{Nonce, NonceSequence, NONCE_LEN};

#[derive(Copy, Clone)]
pub struct InitializedNonceSequence(u128);

impl InitializedNonceSequence {
    pub fn new(iv: [u8; NONCE_LEN]) -> Self {
        let mut bytes = [0u8; 16];
        iv.into_iter()
            .enumerate()
            .for_each(|(i, b)| bytes[i + 4] = b);
        Self(u128::from_be_bytes(bytes))
    }

    // Gets the current nonce so it can be added to ciphertext. This will unwrap the
    // result of `try_into`, which will only fail if the nonce is an invalid u128.
    // This *should* never happen, since [u8; 12] will always be less than a u128
    pub fn current(&self) -> [u8; 12] {
        self.0.to_be_bytes()[4..].try_into().unwrap()
    }
}

impl NonceSequence for InitializedNonceSequence {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        // Use the current value of the counter as the nonce
        let nonce = Nonce::try_assume_unique_for_key(&self.current())?;
        // Increase the counter for the next invocation.
        // 79228162514264337593543950336 = 2^96, the total number of possible nonces
        self.0 = (self.0 + 1) % 79228162514264337593543950336u128;
        Ok(nonce)
    }
}
