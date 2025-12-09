use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce
};
use keyring::Entry;
use rand::RngCore;
use crate::shared::error::{AppError, AppResult};

const ENCRYPTION_SERVICE: &str = "productivity-widgets-db-key";
const ENCRYPTION_KEY_ID: &str = "master_key";

/// Encryption manager that handles key retrieval/generation and encryption/decryption
pub struct EncryptionManager {
    cipher: XChaCha20Poly1305,
}

impl EncryptionManager {
    /// Initialize encryption manager (gets/creates key from keyring)
    pub fn new() -> AppResult<Self> {
        let key_bytes = Self::get_or_create_key()?;
        let cipher = XChaCha20Poly1305::new_from_slice(&key_bytes)
            .map_err(|e| AppError::System(format!("Failed to create cipher: {}", e)))?;
        
        Ok(Self { cipher })
    }

    /// Encrypt data. Returns [Nonce + Ciphertext]
    pub fn encrypt(&self, data: &[u8]) -> AppResult<Vec<u8>> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, data)
            .map_err(|e| AppError::Calculation(format!("Encryption failed: {}", e)))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend(ciphertext);
        
        Ok(result)
    }

    /// Decrypt data. Expects [Nonce + Ciphertext]
    pub fn decrypt(&self, data: &[u8]) -> AppResult<Vec<u8>> {
        if data.len() < 24 { // XChaCha20 nonce is 24 bytes
            return Err(AppError::Validation("Invalid encrypted data length".to_string()));
        }

        let nonce = XNonce::from_slice(&data[0..24]);
        let ciphertext = &data[24..];

        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Calculation(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Retrieve master key from keyring or generate a new one
    fn get_or_create_key() -> AppResult<Vec<u8>> {
        let entry = Entry::new(ENCRYPTION_SERVICE, ENCRYPTION_KEY_ID)
            .map_err(|e| AppError::System(format!("Keyring error: {}", e)))?;

        match entry.get_password() {
            Ok(hex_key) => {
                // Decode existing key
                hex::decode(hex_key).map_err(|e| AppError::Validation(format!("Corrupt master key: {}", e)))
            }
            Err(keyring::Error::NoEntry) => {
                // Generate new 32-byte key
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);
                
                // Store as hex in keyring
                let hex_key = hex::encode(key);
                entry.set_password(&hex_key)
                    .map_err(|e| AppError::System(format!("Failed to save master key: {}", e)))?;
                
                Ok(key.to_vec())
            }
            Err(e) => Err(AppError::System(format!("Keyring access failed: {}", e)))
        }
    }
}
