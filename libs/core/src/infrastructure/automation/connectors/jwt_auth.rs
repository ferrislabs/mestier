use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::json;

pub const JWT_LIFETIME_SECONDS: u64 = 300;

pub fn sign(algorithm: &str, key: &str, issued_at: u64) -> Result<String, String> {
    let (algorithm, encoding) = match algorithm {
        "HS256" => (Algorithm::HS256, EncodingKey::from_secret(key.as_bytes())),
        "HS384" => (Algorithm::HS384, EncodingKey::from_secret(key.as_bytes())),
        "HS512" => (Algorithm::HS512, EncodingKey::from_secret(key.as_bytes())),
        "RS256" => (
            Algorithm::RS256,
            EncodingKey::from_rsa_pem(key.as_bytes())
                .map_err(|error| format!("the JWT private key is not a usable RSA key: {error}"))?,
        ),
        "ES256" => (
            Algorithm::ES256,
            EncodingKey::from_ec_pem(key.as_bytes())
                .map_err(|error| format!("the JWT private key is not a usable EC key: {error}"))?,
        ),
        other => return Err(format!("unsupported JWT algorithm `{other}`")),
    };
    let claims = json!({ "iat": issued_at, "exp": issued_at + JWT_LIFETIME_SECONDS });
    encode(&Header::new(algorithm), &claims, &encoding)
        .map_err(|error| format!("signing the JWT failed: {error}"))
}

#[cfg(test)]
mod tests {
    use aws_lc_rs::{
        encoding::{AsDer, Pkcs8V1Der},
        rand::SystemRandom,
        rsa::{KeyPair as RsaKeyPair, KeySize},
        signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair},
    };
    use base64::{Engine, engine::general_purpose::STANDARD};
    use jsonwebtoken::{DecodingKey, Validation, decode};
    use serde_json::Value;

    use super::*;

    fn pem(label: &str, der: &[u8]) -> String {
        let body = STANDARD.encode(der);
        let lines: Vec<&str> = body
            .as_bytes()
            .chunks(64)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();
        format!(
            "-----BEGIN {label}-----\n{}\n-----END {label}-----\n",
            lines.join("\n")
        )
    }

    fn now() -> u64 {
        chrono::Utc::now().timestamp() as u64
    }

    fn verify(token: &str, algorithm: Algorithm, key: &DecodingKey) -> Value {
        let mut validation = Validation::new(algorithm);
        validation.required_spec_claims.clear();
        decode::<Value>(token, key, &validation).unwrap().claims
    }

    #[test]
    fn the_hmac_algorithms_sign_a_token_the_shared_secret_verifies() {
        for (name, algorithm) in [
            ("HS256", Algorithm::HS256),
            ("HS384", Algorithm::HS384),
            ("HS512", Algorithm::HS512),
        ] {
            let issued_at = now();

            let token = sign(name, "shared-secret", issued_at).unwrap();

            let claims = verify(
                &token,
                algorithm,
                &DecodingKey::from_secret(b"shared-secret"),
            );
            assert_eq!(claims["iat"], issued_at);
            assert_eq!(claims["exp"], issued_at + JWT_LIFETIME_SECONDS);
        }
    }

    #[test]
    fn a_token_signed_with_another_secret_does_not_verify() {
        let token = sign("HS256", "shared-secret", now()).unwrap();
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear();

        assert!(decode::<Value>(&token, &DecodingKey::from_secret(b"other"), &validation).is_err());
    }

    #[test]
    fn rs256_signs_with_a_pem_private_key() {
        let pair = RsaKeyPair::generate(KeySize::Rsa2048).unwrap();
        let private = pem(
            "PRIVATE KEY",
            AsDer::<Pkcs8V1Der>::as_der(&pair).unwrap().as_ref(),
        );

        let token = sign("RS256", &private, now()).unwrap();

        let claims = verify(
            &token,
            Algorithm::RS256,
            &DecodingKey::from_rsa_der(pair.public_key().as_ref()),
        );
        assert!(claims["exp"].as_u64().is_some());
    }

    #[test]
    fn es256_signs_with_a_pem_private_key() {
        let rng = SystemRandom::new();
        let document =
            EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
        let pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, document.as_ref()).unwrap();
        let private = pem("PRIVATE KEY", document.as_ref());

        let token = sign("ES256", &private, now()).unwrap();

        let claims = verify(
            &token,
            Algorithm::ES256,
            &DecodingKey::from_ec_der(pair.public_key().as_ref()),
        );
        assert!(claims["iat"].as_u64().is_some());
    }

    #[test]
    fn a_private_key_that_is_not_a_pem_is_refused_in_a_message_naming_the_algorithm() {
        let error = sign("RS256", "not a key", now()).unwrap_err();
        assert!(error.contains("RSA"), "{error}");
        let error = sign("ES256", "not a key", now()).unwrap_err();
        assert!(error.contains("EC"), "{error}");
    }

    #[test]
    fn an_unknown_algorithm_is_refused() {
        assert!(sign("none", "k", now()).unwrap_err().contains("none"));
    }
}
