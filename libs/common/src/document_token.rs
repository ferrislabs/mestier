use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DocumentTokenError {
	#[error("malformed token")]
	Malformed,
	#[error("token expired")]
	Expired,
	#[error("invalid signature")]
	InvalidSignature,
}

/// Sign a document access token.
///
/// Format: `{doc_id}.{expires_at_unix}.{hex_hmac_sha256}`
///
/// The HMAC is computed over the UTF-8 bytes of `"{doc_id}:{expires_at_unix}"`.
pub fn sign_document_token(doc_id: Uuid, expires_at_unix: i64, secret: &[u8]) -> String {
	let message = format!("{doc_id}:{expires_at_unix}");
	let sig = compute_hmac(secret, message.as_bytes());
	format!("{doc_id}.{expires_at_unix}.{}", hex::encode(sig))
}

/// Verify a document access token.
///
/// Returns the `doc_id` on success.
/// Errors: [`DocumentTokenError::Malformed`], [`DocumentTokenError::Expired`],
/// [`DocumentTokenError::InvalidSignature`].
pub fn verify_document_token(
	token: &str,
	secret: &[u8],
	now_unix: i64,
) -> Result<Uuid, DocumentTokenError> {
	let parts: Vec<&str> = token.splitn(3, '.').collect();
	if parts.len() != 3 {
		return Err(DocumentTokenError::Malformed);
	}

	let doc_id = Uuid::parse_str(parts[0]).map_err(|_| DocumentTokenError::Malformed)?;
	let expires_at: i64 = parts[1].parse().map_err(|_| DocumentTokenError::Malformed)?;
	let provided_sig = hex::decode(parts[2]).map_err(|_| DocumentTokenError::Malformed)?;

	// Recompute HMAC
	let message = format!("{doc_id}:{expires_at}");
	let expected_sig = compute_hmac(secret, message.as_bytes());

	// Constant-time comparison
	if !constant_time_eq(&expected_sig, &provided_sig) {
		return Err(DocumentTokenError::InvalidSignature);
	}

	// Check expiry only after signature is valid
	if now_unix > expires_at {
		return Err(DocumentTokenError::Expired);
	}

	Ok(doc_id)
}

fn compute_hmac(secret: &[u8], message: &[u8]) -> Vec<u8> {
	let mut mac =
		HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
	mac.update(message);
	mac.finalize().into_bytes().to_vec()
}

/// Constant-time byte-slice equality to resist timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
	if a.len() != b.len() {
		return false;
	}
	let mut diff: u8 = 0;
	for (x, y) in a.iter().zip(b.iter()) {
		diff |= x ^ y;
	}
	diff == 0
}

#[cfg(test)]
mod tests {
	use super::*;

	const SECRET: &[u8] = b"test-signing-secret-32-bytes!!!";

	#[test]
	fn round_trip_valid_token() {
		let doc_id = Uuid::new_v4();
		let expires = 9_999_999_999i64; // far future
		let token = sign_document_token(doc_id, expires, SECRET);
		let result = verify_document_token(&token, SECRET, 1_000_000_000);
		assert_eq!(result.unwrap(), doc_id);
	}

	#[test]
	fn expired_token_returns_error() {
		let doc_id = Uuid::new_v4();
		let expires = 1_000i64; // already expired
		let token = sign_document_token(doc_id, expires, SECRET);
		let result = verify_document_token(&token, SECRET, 2_000);
		assert_eq!(result.unwrap_err(), DocumentTokenError::Expired);
	}

	#[test]
	fn tampered_signature_returns_error() {
		let doc_id = Uuid::new_v4();
		let expires = 9_999_999_999i64;
		let token = sign_document_token(doc_id, expires, SECRET);
		// Flip the last character of the signature
		let mut tampered = token.clone();
		let last = tampered.pop().unwrap();
		let replacement = if last == 'a' { 'b' } else { 'a' };
		tampered.push(replacement);
		let result = verify_document_token(&tampered, SECRET, 1_000_000_000);
		assert_eq!(result.unwrap_err(), DocumentTokenError::InvalidSignature);
	}

	#[test]
	fn wrong_secret_returns_invalid_signature() {
		let doc_id = Uuid::new_v4();
		let expires = 9_999_999_999i64;
		let token = sign_document_token(doc_id, expires, SECRET);
		let result = verify_document_token(&token, b"different-secret", 1_000_000_000);
		assert_eq!(result.unwrap_err(), DocumentTokenError::InvalidSignature);
	}

	#[test]
	fn malformed_token_missing_parts() {
		let result = verify_document_token("abc.123", SECRET, 0);
		assert_eq!(result.unwrap_err(), DocumentTokenError::Malformed);
	}

	#[test]
	fn malformed_token_invalid_uuid() {
		let result = verify_document_token("not-a-uuid.1234567890.deadbeef", SECRET, 0);
		assert_eq!(result.unwrap_err(), DocumentTokenError::Malformed);
	}

	#[test]
	fn token_format_is_uuid_dot_expiry_dot_hex() {
		let doc_id = Uuid::new_v4();
		let expires = 1_234_567_890i64;
		let token = sign_document_token(doc_id, expires, SECRET);
		let parts: Vec<&str> = token.splitn(3, '.').collect();
		// parts[0] is the UUID (with hyphens, so 5 segments when split by '-')
		// We just verify the structure here
		assert_eq!(parts.len(), 3);
		assert!(Uuid::parse_str(parts[0]).is_ok());
		assert_eq!(parts[1].parse::<i64>().unwrap(), expires);
		// hex decode must succeed
		assert!(hex::decode(parts[2]).is_ok());
	}
}
