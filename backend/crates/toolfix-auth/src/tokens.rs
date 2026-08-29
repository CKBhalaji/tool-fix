//! Refresh-token generation/hashing and PKCE helpers.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

/// Cryptographically random opaque refresh token (never persisted raw).
pub fn generate_refresh_token() -> String {
    use rand::TryRngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.try_fill_bytes(&mut bytes).expect("os rng");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// The only form of a refresh token that touches the database.
pub fn hash_refresh_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// S256 PKCE pair for the Google OAuth flow.
pub fn generate_pkce() -> (String, String) {
    use rand::TryRngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.try_fill_bytes(&mut bytes).expect("os rng");
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

pub fn generate_state() -> String {
    use rand::TryRngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.try_fill_bytes(&mut bytes).expect("os rng");
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_hash_is_stable_and_differs_per_token() {
        let a = generate_refresh_token();
        let b = generate_refresh_token();
        assert_ne!(a, b);
        assert_eq!(hash_refresh_token(&a), hash_refresh_token(&a));
        assert_ne!(hash_refresh_token(&a), hash_refresh_token(&b));
    }

    #[test]
    fn pkce_challenge_matches_verifier() {
        let (verifier, challenge) = generate_pkce();
        let derived = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        assert_eq!(challenge, derived);
    }
}
