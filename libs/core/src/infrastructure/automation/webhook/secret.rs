use aws_lc_rs::{
    aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};
use common::CoreError;

use crate::domain::automation::endpoint::SealedSecret;

/// Encrypts webhook secrets at rest.
///
/// AES-256-GCM rather than a bare cipher: the tag is what tells decryption
/// apart from garbage, so a tampered or truncated ciphertext fails loudly
/// instead of yielding a plausible-looking key that would then be used to sign
/// somebody's payload.
pub struct SecretCipher {
    key: LessSafeKey,
    rng: SystemRandom,
}

impl SecretCipher {
    pub fn new(key: &[u8; 32]) -> Result<Self, CoreError> {
        let unbound = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| CoreError::Internal("the automation secret key is unusable".to_owned()))?;

        Ok(Self {
            key: LessSafeKey::new(unbound),
            rng: SystemRandom::new(),
        })
    }

    /// Builds a cipher from the base64 key an operator configured.
    ///
    /// Rejects a key of the wrong length rather than padding or truncating
    /// one: a silently shortened key would seal secrets nobody can explain
    /// later.
    pub fn from_base64(encoded: &str) -> Result<Self, CoreError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let decoded = STANDARD.decode(encoded.trim()).map_err(|_| {
            CoreError::Internal("the automation secret key is not valid base64".to_owned())
        })?;

        let key: [u8; 32] = decoded.as_slice().try_into().map_err(|_| {
            CoreError::Internal(format!(
                "the automation secret key must decode to 32 bytes, got {}",
                decoded.len()
            ))
        })?;

        Self::new(&key)
    }

    pub fn seal(&self, plaintext: &[u8]) -> Result<SealedSecret, CoreError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| CoreError::Internal("the system random source failed".to_owned()))?;

        let mut in_out = plaintext.to_vec();
        self.key
            .seal_in_place_append_tag(
                Nonce::assume_unique_for_key(nonce_bytes),
                Aad::empty(),
                &mut in_out,
            )
            .map_err(|_| CoreError::Internal("sealing the webhook secret failed".to_owned()))?;

        Ok(SealedSecret {
            nonce: nonce_bytes.to_vec(),
            ciphertext: in_out,
        })
    }

    pub fn open(&self, sealed: &SealedSecret) -> Result<Vec<u8>, CoreError> {
        let nonce_bytes: [u8; NONCE_LEN] =
            sealed.nonce.as_slice().try_into().map_err(|_| {
                CoreError::Internal("the stored nonce has the wrong length".to_owned())
            })?;

        let mut in_out = sealed.ciphertext.clone();
        let plaintext = self
            .key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce_bytes),
                Aad::empty(),
                &mut in_out,
            )
            .map_err(|_| {
                CoreError::Internal("the stored webhook secret failed authentication".to_owned())
            })?;

        Ok(plaintext.to_vec())
    }

    /// Generates the secret shown once at endpoint creation.
    pub fn generate_secret(&self) -> Result<Vec<u8>, CoreError> {
        let mut secret = vec![0u8; 32];
        self.rng
            .fill(&mut secret)
            .map_err(|_| CoreError::Internal("the system random source failed".to_owned()))?;

        Ok(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cipher() -> SecretCipher {
        SecretCipher::new(&[7u8; 32]).unwrap()
    }

    #[test]
    fn a_configured_key_of_the_right_size_is_accepted() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let encoded = STANDARD.encode([4u8; 32]);

        assert!(SecretCipher::from_base64(&encoded).is_ok());
    }

    /// Padding or truncating a mis-sized key would seal secrets that nobody
    /// can account for afterwards.
    #[test]
    fn a_key_of_the_wrong_size_is_refused_rather_than_adjusted() {
        use base64::{Engine, engine::general_purpose::STANDARD};

        assert!(SecretCipher::from_base64(&STANDARD.encode([4u8; 16])).is_err());
        assert!(SecretCipher::from_base64(&STANDARD.encode([4u8; 64])).is_err());
    }

    #[test]
    fn a_key_that_is_not_base64_is_refused() {
        assert!(SecretCipher::from_base64("not base64 at all !!").is_err());
    }

    #[test]
    fn a_sealed_secret_comes_back_intact() {
        let cipher = cipher();
        let sealed = cipher.seal(b"whsec_abcdef").unwrap();

        assert_eq!(cipher.open(&sealed).unwrap(), b"whsec_abcdef");
    }

    /// Reusing a nonce under one key discloses the keystream, so two seals of
    /// the same secret must never look alike.
    #[test]
    fn sealing_the_same_secret_twice_never_produces_the_same_bytes() {
        let cipher = cipher();

        let first = cipher.seal(b"same secret").unwrap();
        let second = cipher.seal(b"same secret").unwrap();

        assert_ne!(first.nonce, second.nonce);
        assert_ne!(first.ciphertext, second.ciphertext);
    }

    /// The tag is what separates decryption from garbage. Without it a
    /// tampered ciphertext would yield a plausible key and go on to sign
    /// somebody's payload.
    #[test]
    fn a_tampered_ciphertext_fails_authentication() {
        let cipher = cipher();
        let mut sealed = cipher.seal(b"whsec_abcdef").unwrap();
        sealed.ciphertext[0] ^= 0xff;

        assert!(cipher.open(&sealed).is_err());
    }

    #[test]
    fn another_key_cannot_open_it() {
        let sealed = cipher().seal(b"whsec_abcdef").unwrap();
        let other = SecretCipher::new(&[9u8; 32]).unwrap();

        assert!(other.open(&sealed).is_err());
    }

    #[test]
    fn a_truncated_nonce_is_an_error_not_a_panic() {
        let cipher = cipher();
        let mut sealed = cipher.seal(b"whsec_abcdef").unwrap();
        sealed.nonce.truncate(4);

        assert!(cipher.open(&sealed).is_err());
    }

    #[test]
    fn generated_secrets_are_long_and_never_repeat() {
        let cipher = cipher();

        let first = cipher.generate_secret().unwrap();
        let second = cipher.generate_secret().unwrap();

        assert_eq!(first.len(), 32);
        assert_ne!(first, second);
    }
}
