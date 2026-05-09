//! Encryption support (AES-256-GCM)

use crate::error::{Error, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use hkdf::Hkdf;
use sha2::Sha256;

/// Encryption configuration
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Master key (32 bytes for AES-256)
    pub key: Vec<u8>,
    /// Key derivation salt (optional, will be generated if not provided)
    pub salt: Option<Vec<u8>>,
}

impl EncryptionConfig {
    /// Create with key
    pub fn new(key: Vec<u8>) -> Result<Self> {
        if key.len() != 32 {
            return Err(Error::ConfigError(
                "Encryption key must be 32 bytes (256 bits)".to_string(),
            ));
        }
        Ok(EncryptionConfig { key, salt: None })
    }

    /// Create with key and salt
    pub fn with_salt(key: Vec<u8>, salt: Vec<u8>) -> Result<Self> {
        if key.len() != 32 {
            return Err(Error::ConfigError(
                "Encryption key must be 32 bytes (256 bits)".to_string(),
            ));
        }
        if salt.len() != 32 {
            return Err(Error::ConfigError(
                "Salt must be 32 bytes".to_string(),
            ));
        }
        Ok(EncryptionConfig { key, salt: Some(salt) })
    }

    /// Generate a random key
    pub fn generate_key() -> Vec<u8> {
        Aes256Gcm::generate_key(OsRng).to_vec()
    }

    /// Generate a random salt
    pub fn generate_salt() -> Vec<u8> {
        use aes_gcm::aead::OsRng;
        Aes256Gcm::generate_key(OsRng).to_vec()
    }

    /// Derive encryption key from password using HKDF
    pub fn derive_from_password(password: &str, salt: &[u8]) -> Result<Self> {
        if salt.len() != 32 {
            return Err(Error::ConfigError(
                "Salt must be 32 bytes".to_string(),
            ));
        }

        let hkdf = Hkdf::<Sha256>::new(Some(salt), password.as_bytes());
        let mut key = vec![0u8; 32];
        hkdf.expand(b"qrd-encryption-key", &mut key)
            .map_err(|_| Error::ConfigError("Failed to derive key from password".to_string()))?;

        Ok(EncryptionConfig {
            key,
            salt: Some(salt.to_vec()),
        })
    }
}

/// Encrypt data with AES-256-GCM
pub fn encrypt(data: &[u8], config: &EncryptionConfig) -> Result<Vec<u8>> {
    // Generate a random nonce for each encryption
    let nonce_bytes = Aes256Gcm::generate_nonce(OsRng);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Use the master key directly (in production, you might want key derivation)
    let key = Key::<Aes256Gcm>::from_slice(&config.key);
    let cipher = Aes256Gcm::new(key);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|_| Error::EncryptionError("Failed to encrypt data".to_string()))?;

    // Prepend nonce to ciphertext for decryption
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    // If salt was provided, include it for key derivation verification
    if let Some(ref salt) = config.salt {
        let mut salted_result = salt.clone();
        salted_result.extend_from_slice(&result);
        Ok(salted_result)
    } else {
        Ok(result)
    }
}

/// Decrypt data with AES-256-GCM
pub fn decrypt(data: &[u8], config: &EncryptionConfig) -> Result<Vec<u8>> {
    if data.len() < 12 {
        return Err(Error::EncryptionError("Data too short for decryption".to_string()));
    }

    let (_salt, encrypted_data) = if let Some(ref salt) = config.salt {
        // Data includes salt prefix
        if data.len() < 32 + 12 {
            return Err(Error::EncryptionError("Data too short for salted decryption".to_string()));
        }
        if &data[0..32] != salt.as_slice() {
            return Err(Error::EncryptionError("Salt mismatch".to_string()));
        }
        (&data[0..32], &data[32..])
    } else {
        // No salt, data starts with nonce
        (&[] as &[u8], data)
    };

    if encrypted_data.len() < 12 {
        return Err(Error::EncryptionError("Encrypted data too short".to_string()));
    }

    // Extract nonce (first 12 bytes)
    let nonce = Nonce::from_slice(&encrypted_data[0..12]);
    let ciphertext = &encrypted_data[12..];

    // Use the master key
    let key = Key::<Aes256Gcm>::from_slice(&config.key);
    let cipher = Aes256Gcm::new(key);

    // Decrypt the data
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| Error::EncryptionError("Failed to decrypt data - invalid key or corrupted data".to_string()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = EncryptionConfig::generate_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_salt_generation() {
        let salt = EncryptionConfig::generate_salt();
        assert_eq!(salt.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = EncryptionConfig::generate_key();
        let config = EncryptionConfig::new(key).unwrap();

        let original_data = b"Hello, QRD encryption!";
        let encrypted = encrypt(original_data, &config).unwrap();
        let decrypted = decrypt(&encrypted, &config).unwrap();

        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_with_salt() {
        let key = EncryptionConfig::generate_key();
        let salt = EncryptionConfig::generate_salt();
        let config = EncryptionConfig::with_salt(key, salt).unwrap();

        let original_data = b"Salted encryption test";
        let encrypted = encrypt(original_data, &config).unwrap();
        let decrypted = decrypt(&encrypted, &config).unwrap();

        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_password_derivation() {
        let password = "my-secret-password";
        let salt = EncryptionConfig::generate_salt();

        let config = EncryptionConfig::derive_from_password(password, &salt).unwrap();
        assert_eq!(config.key.len(), 32);
        assert_eq!(config.salt.as_ref().unwrap(), &salt);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = EncryptionConfig::generate_key();
        let key2 = EncryptionConfig::generate_key();
        let config1 = EncryptionConfig::new(key1).unwrap();
        let config2 = EncryptionConfig::new(key2).unwrap();

        let original_data = b"Secret data";
        let encrypted = encrypt(original_data, &config1).unwrap();

        // Should fail with wrong key
        let result = decrypt(&encrypted, &config2);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let key = vec![0u8; 16]; // Wrong length
        let result = EncryptionConfig::new(key);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_salt_length() {
        let key = EncryptionConfig::generate_key();
        let salt = vec![0u8; 16]; // Wrong length
        let result = EncryptionConfig::with_salt(key, salt);
        assert!(result.is_err());
    }
}
