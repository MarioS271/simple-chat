// SPDX-License-Identifier: GPL-3.0-only
//! Encryption Logic; Generate Keys and Encrypt/Decrypt Messages
//!
//! Authors: MarioS271

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;

const ENCRYPTION_KEY_BYTES: usize = 32;
const NONCE_BYTES: usize = 12;
const TAG_BYTES: usize = 16;

pub type EncryptionKey = [u8; ENCRYPTION_KEY_BYTES];

pub fn generate_key() -> Result<EncryptionKey, String> {
    let mut key = [0u8; ENCRYPTION_KEY_BYTES];
    getrandom::fill(&mut key).map_err(|e| format!("Could not generate key: {}", e))?;
    Ok(key)
}

pub fn encrypt(key: &EncryptionKey, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(key.into());

    let mut nonce_bytes = [0u8; NONCE_BYTES];
    getrandom::fill(&mut nonce_bytes).map_err(|e| format!("Could not generate nonce: {}", e))?;
    let nonce = Nonce::try_from(nonce_bytes.as_slice())
        .map_err(|_| "Invalid nonce length".to_string())?;

    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|_| "Encryption failed".to_string())?;

    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn decrypt(key: &EncryptionKey, data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < NONCE_BYTES + TAG_BYTES {
        return Err(format!(
            "Data too short (is {} bytes, but needs to be at minimum {} bytes)",
            data.len(),
            NONCE_BYTES + TAG_BYTES
        ));
    }

    let cipher = Aes256Gcm::new(key.into());

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_BYTES);
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|_| "Invalid nonce length".to_string())?;

    cipher.decrypt(&nonce, ciphertext).map_err(|_| "Decryption failed".to_string())
}
