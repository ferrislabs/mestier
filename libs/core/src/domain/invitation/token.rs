//! Invitation token generation and hashing.
//!
//! A pure computation, not an adapter: unlike [`crate::infrastructure::automation::webhook::secret::SecretCipher`],
//! nothing here holds managed key material or reads configuration — the CSPRNG
//! and the digest are both stateless. That is exactly the same footing as
//! [`common::generate_uuid_v7`], already called directly from domain services
//! (e.g. `MemberService::create_member`), so calling `aws_lc_rs` here does not
//! cross the domain/infrastructure boundary the way sealing a webhook secret
//! does.
//!
//! **Why SHA-256, not the HMAC-SHA256 primitive `webhook::signature` uses**:
//! that primitive signs a payload with a *secret key* so a receiver holding
//! the same key can verify authenticity. An invitation token has no
//! counterpart key to share — it is itself 256 bits of CSPRNG entropy, so a
//! keyed hash would either need a fixed, source-visible key (worthless: it
//! provides no more resistance than a keyless hash) or `AUTOMATION_SECRET_KEY`
//! (optional, and this table must not depend on an operator having set it — see
//! `MestierUseCase::require_cipher`, which credentials sealing *does* accept
//! failing without). A keyless digest of a high-entropy token is the same
//! irreversibility property webhook secrets get from HMAC, without adding a
//! dependency invitations do not need.

use aws_lc_rs::{digest, rand::SecureRandom, rand::SystemRandom};
use common::CoreError;

/// Bytes of CSPRNG entropy in the clear token — 256 bits, the same margin
/// `SecretCipher::generate_secret` uses for a webhook secret.
const TOKEN_BYTES: usize = 32;

/// Generates a fresh invitation token: the clear value (hex-encoded, returned
/// to the caller exactly once) and its digest (what gets stored).
pub fn generate() -> Result<(String, Vec<u8>), CoreError> {
    let mut bytes = [0u8; TOKEN_BYTES];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| CoreError::Internal("the system random source failed".to_owned()))?;

    let clear = hex(&bytes);
    let hash = hash(&clear);
    Ok((clear, hash))
}

/// The digest of a presented token, for an exact-match lookup by
/// `token_hash` — see the index in `20260812000001_create_organization_invitations.up.sql`.
/// No separate constant-time comparison is needed on top: this is an
/// equality lookup against an indexed column, not a byte-by-byte comparison
/// against attacker-supplied input, so there is no timing oracle to guard
/// against the way `webhook::signature::verify` does for a value it receives
/// over the wire and compares itself.
pub fn hash(token: &str) -> Vec<u8> {
    digest::digest(&digest::SHA256, token.as_bytes())
        .as_ref()
        .to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_returns_a_clear_token_and_its_hash() {
        let (clear, hash) = generate().unwrap();

        assert_eq!(clear.len(), TOKEN_BYTES * 2);
        assert!(clear.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash.len(), 32); // SHA-256 digest length
    }

    #[test]
    fn generate_never_repeats() {
        let (first_clear, first_hash) = generate().unwrap();
        let (second_clear, second_hash) = generate().unwrap();

        assert_ne!(first_clear, second_clear);
        assert_ne!(first_hash, second_hash);
    }

    #[test]
    fn hash_is_deterministic() {
        let (clear, hash_at_generation) = generate().unwrap();

        assert_eq!(hash(&clear), hash_at_generation);
    }

    #[test]
    fn hash_is_sensitive_to_every_character() {
        let (clear, original_hash) = generate().unwrap();
        let mut tampered = clear.clone();
        tampered.replace_range(0..1, if &clear[0..1] == "0" { "1" } else { "0" });

        assert_ne!(hash(&tampered), original_hash);
    }
}
