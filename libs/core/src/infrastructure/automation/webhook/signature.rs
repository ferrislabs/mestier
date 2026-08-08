use aws_lc_rs::hmac;

/// Signs the body a receiver is about to read.
///
/// The timestamp is **inside** the signature, not merely beside it. Signing
/// the body alone leaves an intercepted call replayable forever; with the
/// timestamp covered, a receiver can reject anything outside its tolerance
/// window and know the value was not moved.
pub fn sign(secret: &[u8], timestamp: i64, body: &[u8]) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    let tag = hmac::sign(&key, &signed_bytes(timestamp, body));

    hex(tag.as_ref())
}

/// The receiver's side of the contract, kept here so the documentation and the
/// implementation cannot drift apart.
pub fn verify(secret: &[u8], timestamp: i64, body: &[u8], signature: &str) -> bool {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    let Some(provided) = unhex(signature) else {
        return false;
    };

    hmac::verify(&key, &signed_bytes(timestamp, body), &provided).is_ok()
}

fn signed_bytes(timestamp: i64, body: &[u8]) -> Vec<u8> {
    let mut signed = timestamp.to_string().into_bytes();
    signed.push(b'.');
    signed.extend_from_slice(body);
    signed
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }

    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4231-style known answer, so this proves the primitive is wired
    /// correctly rather than merely being consistent with itself.
    #[test]
    fn hmac_sha256_matches_a_published_vector() {
        let key = hmac::Key::new(hmac::HMAC_SHA256, b"key");
        let tag = hmac::sign(&key, b"The quick brown fox jumps over the lazy dog");

        assert_eq!(
            hex(tag.as_ref()),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn a_receiver_holding_the_secret_can_verify_what_we_sent() {
        let signature = sign(b"shhh", 1_754_640_000, br#"{"name":"quote.accepted"}"#);

        assert!(verify(
            b"shhh",
            1_754_640_000,
            br#"{"name":"quote.accepted"}"#,
            &signature
        ));
    }

    #[test]
    fn a_body_altered_in_transit_fails_verification() {
        let signature = sign(b"shhh", 1_754_640_000, br#"{"total_cents":100}"#);

        assert!(!verify(
            b"shhh",
            1_754_640_000,
            br#"{"total_cents":999999}"#,
            &signature
        ));
    }

    /// The property that makes a replay detectable: move the timestamp and the
    /// signature no longer matches, so an intercepted call cannot be resent
    /// later under a fresh one.
    #[test]
    fn a_replayed_call_cannot_carry_a_new_timestamp() {
        let body = br#"{"name":"quote.accepted"}"#;
        let signature = sign(b"shhh", 1_754_640_000, body);

        assert!(!verify(b"shhh", 1_754_643_600, body, &signature));
    }

    #[test]
    fn another_secret_does_not_verify() {
        let signature = sign(b"ours", 1_754_640_000, b"payload");

        assert!(!verify(b"theirs", 1_754_640_000, b"payload", &signature));
    }

    #[test]
    fn a_malformed_signature_is_refused_rather_than_panicking() {
        assert!(!verify(b"shhh", 1, b"payload", "not-hex"));
        assert!(!verify(b"shhh", 1, b"payload", "abc"));
        assert!(!verify(b"shhh", 1, b"payload", ""));
    }
}
