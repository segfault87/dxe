use std::string::FromUtf8Error;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

pub struct AesCrypto {
    cipher: Aes256Gcm,
}

impl AesCrypto {
    pub fn new(key: &[u8]) -> Self {
        let key: &Key<Aes256Gcm> = Key::<Aes256Gcm>::from_slice(key);

        Self {
            cipher: Aes256Gcm::new(key),
        }
    }

    pub fn encrypt(&self, nonce: Option<[u8; 12]>, plaintext: &[u8]) -> Result<String, Error> {
        let nonce = Nonce::from(nonce.unwrap_or([0; 12]));

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(Error::Encrypt)?;

        Ok(BASE64_STANDARD.encode(ciphertext))
    }

    pub fn decrypt(
        &self,
        nonce: Option<[u8; 12]>,
        base64_ciphertext: &[u8],
    ) -> Result<String, Error> {
        let nonce = Nonce::from(nonce.unwrap_or([0; 12]));

        let bytes = BASE64_STANDARD.decode(base64_ciphertext)?;
        let plaintext = self
            .cipher
            .decrypt(&nonce, bytes.as_slice())
            .map_err(Error::Decrypt)?;

        Ok(String::from_utf8(plaintext)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Encryption error: {0}")]
    Encrypt(aes_gcm::Error),
    #[error("Decryption error: {0}")]
    Decrypt(aes_gcm::Error),
    #[error("Incorrect base64 string")]
    InvalidBase64String(#[from] base64::DecodeError),
    #[error("Invalid UTF8 string")]
    InvalidUtf8String(#[from] FromUtf8Error),
}
