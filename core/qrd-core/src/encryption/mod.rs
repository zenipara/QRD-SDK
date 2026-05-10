//! Encryption support (AES-256-GCM)

use crate::error::{Error, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
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
            return Err(Error::ConfigError("Salt must be 32 bytes".to_string()));
        }
        Ok(EncryptionConfig {
            key,
            salt: Some(salt),
        })
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
        // Prefer stronger password hashing for user passwords. Delegate to
        // the Argon2-backed helper which is the recommended flow.
        // Keep a compatibility wrapper that enforces Argon2-based derivation.
        Ok(EncryptionConfig::derive_from_user_password(
            password,
            Some(salt),
        )?)
    }

    /// Derive encryption key from user password using Argon2id + HKDF
    ///
    /// This is the recommended method for user-provided passwords.
    /// Uses Argon2id (password hashing) followed by HKDF (key derivation).
    ///
    /// # Arguments
    /// * `password` - User-provided password (any length)
    /// * `argon2_salt` - Salt for Argon2 (16 bytes recommended, will be generated if None)
    ///
    /// # Returns
    /// EncryptionConfig with derived key and Argon2 salt stored
    pub fn derive_from_user_password(password: &str, argon2_salt: Option<&[u8]>) -> Result<Self> {
        // Generate or use provided salt for Argon2
        let (salt_str, argon2_salt_vec) = if let Some(salt_bytes) = argon2_salt {
            // Use provided raw salt bytes for Argon2 and also store original bytes
            let s = SaltString::encode_b64(salt_bytes)
                .map_err(|_| Error::ConfigError("Invalid Argon2 salt".to_string()))?;
            (s, salt_bytes.to_vec())
        } else {
            // Generate a 32-byte raw salt and encode it for Argon2
            let raw_salt = EncryptionConfig::generate_salt();
            let s = SaltString::encode_b64(&raw_salt)
                .map_err(|_| Error::ConfigError("Failed to encode generated salt".to_string()))?;
            (s, raw_salt)
        };

        // Hash password using Argon2id
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_str)
            .map_err(|_| Error::ConfigError("Failed to hash password with Argon2".to_string()))?;

        // Convert hash output to owned bytes for HKDF
        let hash_bytes = password_hash
            .hash
            .ok_or_else(|| Error::ConfigError("Argon2 hash missing output".to_string()))?
            .as_bytes()
            .to_vec();

        // Derive encryption key from hashed password using HKDF
        let hkdf = Hkdf::<Sha256>::new(None, &hash_bytes);
        let mut key = vec![0u8; 32];
        hkdf.expand(b"qrd-encryption-key-from-password", &mut key)
            .map_err(|_| {
                Error::ConfigError("Failed to derive key from password hash".to_string())
            })?;

        // Store the Argon2 salt for key derivation consistency (raw bytes)
        let argon2_salt_vec = argon2_salt_vec;

        Ok(EncryptionConfig {
            key,
            salt: Some(argon2_salt_vec),
        })
    }

    /// Derive a per-column encryption key from the master key
    ///
    /// Uses HKDF with the column name as additional context to derive a unique key
    /// for each column while maintaining deterministic key generation.
    ///
    /// # Arguments
    /// * `column_name` - The name of the column (used as HKDF info string)
    ///
    /// # Returns
    /// A unique 32-byte key derived from the master key and column name
    pub fn derive_column_key(&self, column_name: &str) -> Result<Vec<u8>> {
        let hkdf = Hkdf::<Sha256>::new(None, &self.key);
        let mut column_key = vec![0u8; 32];

        // Use column name as HKDF info to derive unique key per column
        hkdf.expand(
            format!("qrd-column-key:{}", column_name).as_bytes(),
            &mut column_key,
        )
        .map_err(|_| {
            Error::ConfigError(format!("Failed to derive column key for '{}'", column_name))
        })?;

        Ok(column_key)
    }
}

/// Per-column encryption information
///
/// Stores metadata about which columns are encrypted and with which keys
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerColumnEncryption {
    /// Whether per-column encryption is enabled
    pub enabled: bool,
    /// List of column names that are encrypted (in schema order)
    pub encrypted_columns: Vec<String>,
    /// Master salt used for key derivation (if applicable)
    pub master_salt: Option<Vec<u8>>,
}

impl PerColumnEncryption {
    /// Create a new per-column encryption config
    pub fn new(master_salt: Option<Vec<u8>>) -> Self {
        PerColumnEncryption {
            enabled: true,
            encrypted_columns: Vec::new(),
            master_salt,
        }
    }

    /// Check if a column is encrypted
    pub fn is_column_encrypted(&self, column_name: &str) -> bool {
        self.enabled && self.encrypted_columns.contains(&column_name.to_string())
    }

    /// Mark a column as encrypted
    pub fn mark_column_encrypted(&mut self, column_name: String) {
        if self.enabled && !self.encrypted_columns.contains(&column_name) {
            self.encrypted_columns.push(column_name);
        }
    }
}

/// Encrypt data with AES-256-GCM

///
/// Output format (standardized):
/// [1B flags][optional 32B salt][12B nonce][ciphertext][16B GCM tag]
///
/// flags byte:
///   bit 0: has_salt (1 = ada salt, 0 = tanpa salt)
///   bit 1-7: reserved (harus 0)
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

    // Build result with flags byte
    let mut result = Vec::new();

    // Flags byte: bit 0 = has_salt
    let flags: u8 = if config.salt.is_some() { 0x01 } else { 0x00 };
    result.push(flags);

    // Optional salt (if present)
    if let Some(ref salt) = config.salt {
        result.extend_from_slice(salt);
    }

    // Nonce (12 bytes)
    result.extend_from_slice(&nonce_bytes);

    // Ciphertext (includes GCM tag)
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data with AES-256-GCM
///
/// Expects format: [1B flags][optional 32B salt][12B nonce][ciphertext][16B GCM tag]
pub fn decrypt(data: &[u8], config: &EncryptionConfig) -> Result<Vec<u8>> {
    if data.len() < 1 {
        return Err(Error::EncryptionError(
            "Encrypted data too short - missing flags byte".to_string(),
        ));
    }

    let flags = data[0];
    let has_salt = (flags & 0x01) != 0;

    // Validate reserved bits
    if (flags & 0xFE) != 0 {
        return Err(Error::EncryptionError(
            "Invalid flags byte - reserved bits set".to_string(),
        ));
    }

    let mut offset = 1; // After flags byte

    // Parse optional salt
    if has_salt {
        if offset + 32 > data.len() {
            return Err(Error::EncryptionError(
                "Encrypted data too short - truncated salt".to_string(),
            ));
        }

        let stored_salt = &data[offset..offset + 32];
        if let Some(ref config_salt) = config.salt {
            if stored_salt != config_salt.as_slice() {
                return Err(Error::EncryptionError("Salt mismatch".to_string()));
            }
        } else {
            return Err(Error::EncryptionError(
                "Encrypted data has salt but config doesn't expect one".to_string(),
            ));
        }

        offset += 32;
    } else {
        if config.salt.is_some() {
            return Err(Error::EncryptionError(
                "Encrypted data has no salt but config expects one".to_string(),
            ));
        }
    }

    // Parse nonce (12 bytes)
    if offset + 12 > data.len() {
        return Err(Error::EncryptionError(
            "Encrypted data too short - truncated nonce".to_string(),
        ));
    }

    let nonce = Nonce::from_slice(&data[offset..offset + 12]);
    offset += 12;

    // The rest is ciphertext + GCM tag
    let ciphertext_with_tag = &data[offset..];

    if ciphertext_with_tag.is_empty() {
        return Err(Error::EncryptionError(
            "Encrypted data has no ciphertext".to_string(),
        ));
    }

    // Use the master key
    let key = Key::<Aes256Gcm>::from_slice(&config.key);
    let cipher = Aes256Gcm::new(key);

    // Decrypt the data
    let plaintext = cipher.decrypt(nonce, ciphertext_with_tag).map_err(|_| {
        Error::EncryptionError("Failed to decrypt data - invalid key or corrupted data".to_string())
    })?;

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

    #[test]
    fn test_per_column_key_derivation() {
        let key = EncryptionConfig::generate_key();
        let config = EncryptionConfig::new(key).unwrap();

        // Derive keys for two columns
        let col1_key = config.derive_column_key("temperature").unwrap();
        let col2_key = config.derive_column_key("humidity").unwrap();

        // Keys should be different
        assert_ne!(col1_key, col2_key);
        // But deterministic (same input = same output)
        let col1_key_again = config.derive_column_key("temperature").unwrap();
        assert_eq!(col1_key, col1_key_again);
        // All keys should be 32 bytes
        assert_eq!(col1_key.len(), 32);
        assert_eq!(col2_key.len(), 32);
    }

    #[test]
    fn test_per_column_encryption_metadata() {
        let mut pc_enc = PerColumnEncryption::new(None);

        assert!(pc_enc.enabled);
        assert!(!pc_enc.is_column_encrypted("temperature"));

        pc_enc.mark_column_encrypted("temperature".to_string());
        assert!(pc_enc.is_column_encrypted("temperature"));
        assert!(!pc_enc.is_column_encrypted("humidity"));

        pc_enc.mark_column_encrypted("humidity".to_string());
        assert_eq!(pc_enc.encrypted_columns.len(), 2);
    }

    #[test]
    fn test_per_column_key_encrypt_decrypt() {
        let key = EncryptionConfig::generate_key();
        let config = EncryptionConfig::new(key).unwrap();

        let col1_key = config.derive_column_key("temperature").unwrap();
        let col2_key = config.derive_column_key("humidity").unwrap();

        // Create configs with derived keys
        let col1_config = EncryptionConfig::new(col1_key).unwrap();
        let col2_config = EncryptionConfig::new(col2_key).unwrap();

        let data1 = b"23.5";
        let data2 = b"65";

        // Encrypt each column with its own key
        let encrypted1 = encrypt(data1, &col1_config).unwrap();
        let encrypted2 = encrypt(data2, &col2_config).unwrap();

        // Decrypt with correct keys
        let decrypted1 = decrypt(&encrypted1, &col1_config).unwrap();
        let decrypted2 = decrypt(&encrypted2, &col2_config).unwrap();

        assert_eq!(data1, decrypted1.as_slice());
        assert_eq!(data2, decrypted2.as_slice());

        // Should fail with wrong key
        assert!(decrypt(&encrypted1, &col2_config).is_err());
        assert!(decrypt(&encrypted2, &col1_config).is_err());
    }
}
