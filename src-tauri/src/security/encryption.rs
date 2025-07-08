use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use anyhow::{Context, Result};

/// Encrypts data using AES-256-GCM (recommended for most use cases)
pub fn encrypt_data(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::generate(&mut OsRng);
    
    cipher.encrypt(&nonce, data)
        .map(|mut ciphertext| {
            // Prepend nonce to ciphertext for storage
            ciphertext.splice(0..0, nonce.iter().copied());
            ciphertext
        })
        .context("Encryption failed")
}

/// Decrypts data using AES-256-GCM
pub fn decrypt_data(encrypted_data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    // Extract nonce (first 12 bytes)
    if encrypted_data.len() < 12 {
        anyhow::bail!("Invalid encrypted data length");
    }
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let cipher = Aes256Gcm::new_from_slice(key)?;
    
    cipher.decrypt(nonce, &encrypted_data[12..])
        .context("Decryption failed")
}

/// Generates a secure random key
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}