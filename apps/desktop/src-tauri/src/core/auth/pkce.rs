use std::sync::Mutex;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;

/// PKCE State Manager
/// Stores the code_verifier in memory for the duration of the auth flow.
/// CRITICAL: The verifier MUST NEVER leave this application or be sent over the network.
pub struct PkceState {
    verifier: Mutex<Option<String>>,
}

impl PkceState {
    /// Create a new PKCE state manager
    pub fn new() -> Self {
        Self {
            verifier: Mutex::new(None),
        }
    }

    /// Generate PKCE proof (verifier + challenge)
    /// Returns: (verifier, challenge)
    /// 
    /// The verifier is a cryptographically random 32-byte string.
    /// The challenge is the URL-safe Base64 encoding of SHA256(verifier).
    pub fn generate_proof(&self) -> Result<(String, String), String> {
        // Generate 32 random bytes for the verifier
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        
        // Convert to URL-safe base64 (no padding)
        let verifier = URL_SAFE_NO_PAD.encode(&random_bytes);
        
        // Compute SHA256 hash of the verifier
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        
        // Encode challenge as URL-safe base64 (CRITICAL: URL_SAFE_NO_PAD to avoid + and / in URLs)
        let challenge = URL_SAFE_NO_PAD.encode(&hash);
        
        // Store verifier in state
        self.store_verifier(verifier.clone())?;
        
        Ok((verifier, challenge))
    }

    /// Store the verifier in memory
    fn store_verifier(&self, verifier: String) -> Result<(), String> {
        let mut guard = self.verifier.lock()
            .map_err(|e| format!("Failed to lock verifier mutex: {}", e))?;
        *guard = Some(verifier);
        Ok(())
    }

    /// Retrieve and consume the verifier (one-time use)
    pub fn get_and_clear_verifier(&self) -> Result<String, String> {
        let mut guard = self.verifier.lock()
            .map_err(|e| format!("Failed to lock verifier mutex: {}", e))?;
        
        guard.take()
            .ok_or_else(|| "No verifier found. Auth flow not initiated.".to_string())
    }

    /// Clear the verifier without returning it
    pub fn clear_verifier(&self) -> Result<(), String> {
        let mut guard = self.verifier.lock()
            .map_err(|e| format!("Failed to lock verifier mutex: {}", e))?;
        *guard = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_proof() {
        let state = PkceState::new();
        let result = state.generate_proof();
        assert!(result.is_ok());
        
        let (verifier, challenge) = result.unwrap();
        
        // Verifier should be base64-encoded 32 bytes
        assert!(!verifier.is_empty());
        
        // Challenge should be base64-encoded SHA256 hash (32 bytes)
        assert!(!challenge.is_empty());
        
        // Verify challenge is correct
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let expected_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
        assert_eq!(challenge, expected_challenge);
    }

    #[test]
    fn test_get_and_clear_verifier() {
        let state = PkceState::new();
        
        // Generate proof to store verifier
        let (verifier, _) = state.generate_proof().unwrap();
        
        // Retrieve verifier
        let retrieved = state.get_and_clear_verifier().unwrap();
        assert_eq!(verifier, retrieved);
        
        // Second retrieval should fail (one-time use)
        let result = state.get_and_clear_verifier();
        assert!(result.is_err());
    }

    #[test]
    fn test_url_safe_encoding() {
        let state = PkceState::new();
        let (_, challenge) = state.generate_proof().unwrap();
        
        // Verify no + or / characters (URL-unsafe)
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));  // No padding
    }
}
