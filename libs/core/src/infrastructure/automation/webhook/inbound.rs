use aws_lc_rs::{constant_time, digest, hmac};
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::infrastructure::automation::connectors::http_client::HttpAuth;

#[cfg(test)]
thread_local! {
    static COMPARISONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub fn constant_time_equals(left: &[u8], right: &[u8]) -> bool {
    #[cfg(test)]
    COMPARISONS.with(|count| count.set(count.get() + 1));
    let left = digest::digest(&digest::SHA256, left);
    let right = digest::digest(&digest::SHA256, right);
    constant_time::verify_slices_are_equal(left.as_ref(), right.as_ref()).is_ok()
}

pub fn path_fingerprint(path: &str) -> String {
    digest::digest(&digest::SHA256, path.as_bytes())
        .as_ref()
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn holds_a_blank_secret(expected: &HttpAuth) -> bool {
    let blank = |text: &str| text.trim().is_empty();
    match expected {
        HttpAuth::Bearer { token } => blank(token),
        HttpAuth::Basic { username, password } => blank(username) || blank(password),
        HttpAuth::Header { name, value } => blank(name) || blank(value),
        _ => true,
    }
}

pub fn presents<'a>(expected: &HttpAuth, header: impl Fn(&str) -> Option<&'a str>) -> bool {
    if holds_a_blank_secret(expected) {
        return false;
    }
    match expected {
        HttpAuth::Bearer { token } => header("authorization")
            .and_then(|value| after_scheme(value, "bearer"))
            .is_some_and(|presented| constant_time_equals(presented.as_bytes(), token.as_bytes())),
        HttpAuth::Basic { username, password } => header("authorization")
            .and_then(|value| after_scheme(value, "basic"))
            .and_then(|encoded| STANDARD.decode(encoded).ok())
            .is_some_and(|decoded| {
                constant_time_equals(&decoded, format!("{username}:{password}").as_bytes())
            }),
        HttpAuth::Header { name, value } => header(name)
            .is_some_and(|presented| constant_time_equals(presented.as_bytes(), value.as_bytes())),
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl SignatureAlgorithm {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }

    fn prefix(self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    fn algorithm(self) -> hmac::Algorithm {
        match self {
            Self::Sha1 => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            Self::Sha256 => hmac::HMAC_SHA256,
            Self::Sha512 => hmac::HMAC_SHA512,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureSpec {
    pub header_name: String,
    pub algorithm: SignatureAlgorithm,
    pub secret: String,
}

impl SignatureSpec {
    pub fn is_presented<'a>(&self, header: impl Fn(&str) -> Option<&'a str>) -> bool {
        !self.secret.trim().is_empty()
            && !self.header_name.trim().is_empty()
            && header(&self.header_name).is_some_and(|value| !value.trim().is_empty())
    }

    pub fn sign(&self, body: &[u8]) -> String {
        let key = hmac::Key::new(self.algorithm.algorithm(), self.secret.as_bytes());
        let tag = hmac::sign(&key, body);
        let digest: String = tag
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        format!("{}={digest}", self.algorithm.prefix())
    }

    pub fn verify<'a>(&self, header: impl Fn(&str) -> Option<&'a str>, body: &[u8]) -> bool {
        if !self.is_presented(&header) {
            return false;
        }
        let Some(presented) = header(&self.header_name) else {
            return false;
        };
        let presented = presented.trim();
        let encoded = match presented.split_once('=') {
            Some((prefix, rest)) if prefix.eq_ignore_ascii_case(self.algorithm.prefix()) => rest,
            Some(_) => return false,
            None => presented,
        };
        let Some(expected) = decode_hex(encoded) else {
            return false;
        };
        let key = hmac::Key::new(self.algorithm.algorithm(), self.secret.as_bytes());
        hmac::verify(&key, body, &expected).is_ok()
    }
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.is_empty() || !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|index| {
            text.get(index..index + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        })
        .collect()
}

fn after_scheme<'a>(value: &'a str, scheme: &str) -> Option<&'a str> {
    let (presented, rest) = value.split_once(' ')?;
    presented.eq_ignore_ascii_case(scheme).then_some(rest)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<&'static str> {
        let map: HashMap<String, &'static str> = pairs
            .iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), *value))
            .collect();
        move |name| map.get(&name.to_ascii_lowercase()).copied()
    }

    fn bearer() -> HttpAuth {
        HttpAuth::Bearer {
            token: "s3cret".to_owned(),
        }
    }

    fn basic() -> HttpAuth {
        HttpAuth::Basic {
            username: "alice".to_owned(),
            password: "p:ss word".to_owned(),
        }
    }

    fn header_auth() -> HttpAuth {
        HttpAuth::Header {
            name: "X-Api-Key".to_owned(),
            value: "k3y".to_owned(),
        }
    }

    fn spec(algorithm: SignatureAlgorithm) -> SignatureSpec {
        SignatureSpec {
            header_name: "X-Signature".to_owned(),
            algorithm,
            secret: "shared-secret".to_owned(),
        }
    }

    fn signed(algorithm: hmac::Algorithm, body: &[u8]) -> String {
        let tag = hmac::sign(&hmac::Key::new(algorithm, b"shared-secret"), body);
        tag.as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn a_body_signed_with_the_shared_secret_is_authenticated_for_each_algorithm() {
        let body = br#"{"montant":12}"#;
        for (algorithm, primitive) in [
            (
                SignatureAlgorithm::Sha1,
                hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            ),
            (SignatureAlgorithm::Sha256, hmac::HMAC_SHA256),
            (SignatureAlgorithm::Sha512, hmac::HMAC_SHA512),
        ] {
            let signature = signed(primitive, body);

            assert!(
                spec(algorithm).verify(header_with("X-Signature", &signature), body),
                "{algorithm:?}"
            );
        }
    }

    fn header_with(name: &str, value: &str) -> impl Fn(&str) -> Option<&'static str> {
        headers_owned(vec![(name.to_owned(), value.to_owned())])
    }

    fn headers_owned(pairs: Vec<(String, String)>) -> impl Fn(&str) -> Option<&'static str> {
        let leaked: Vec<(String, &'static str)> = pairs
            .into_iter()
            .map(|(name, value)| {
                (
                    name.to_ascii_lowercase(),
                    &*Box::leak(value.into_boxed_str()),
                )
            })
            .collect();
        move |name| {
            leaked
                .iter()
                .find(|(candidate, _)| *candidate == name.to_ascii_lowercase())
                .map(|(_, value)| *value)
        }
    }

    #[test]
    fn the_signature_may_carry_the_algorithm_prefix_in_any_case_and_the_hex_in_any_case() {
        let body = b"payload";
        let signature = signed(hmac::HMAC_SHA256, body);

        for presented in [
            format!("sha256={signature}"),
            format!("SHA256={signature}"),
            signature.to_ascii_uppercase(),
            format!("  {signature}  "),
        ] {
            assert!(
                spec(SignatureAlgorithm::Sha256).verify(
                    headers_owned(vec![("X-Signature".to_owned(), presented.clone())]),
                    body
                ),
                "{presented}"
            );
        }
    }

    #[test]
    fn a_wrong_secret_body_algorithm_prefix_or_encoding_is_refused() {
        let body = b"payload";
        let signature = signed(hmac::HMAC_SHA256, body);
        let other_secret = {
            let tag = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, b"another-secret"), body);
            tag.as_ref()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };

        for (presented, sent_body) in [
            (other_secret.clone(), body.to_vec()),
            (signature.clone(), b"payloae".to_vec()),
            (signature.clone(), Vec::new()),
            (format!("sha1={signature}"), body.to_vec()),
            (signature[..signature.len() - 2].to_owned(), body.to_vec()),
            (format!("{signature}00"), body.to_vec()),
            ("not-hex".to_owned(), body.to_vec()),
            ("zz".repeat(32), body.to_vec()),
            (String::new(), body.to_vec()),
            ("sha256=".to_owned(), body.to_vec()),
        ] {
            assert!(
                !spec(SignatureAlgorithm::Sha256).verify(
                    headers_owned(vec![("X-Signature".to_owned(), presented.clone())]),
                    &sent_body
                ),
                "{presented}"
            );
        }
        assert!(!spec(SignatureAlgorithm::Sha512).verify(
            headers_owned(vec![("X-Signature".to_owned(), signature)]),
            body
        ));
    }

    #[test]
    fn a_missing_header_or_a_blank_secret_authenticates_nothing() {
        let body = b"payload";
        let signature = signed(hmac::HMAC_SHA256, body);
        let blank_secret = SignatureSpec {
            secret: "  ".to_owned(),
            ..spec(SignatureAlgorithm::Sha256)
        };
        let blank_header = SignatureSpec {
            header_name: " ".to_owned(),
            ..spec(SignatureAlgorithm::Sha256)
        };

        assert!(!spec(SignatureAlgorithm::Sha256).verify(headers(&[]), body));
        assert!(!spec(SignatureAlgorithm::Sha256).verify(header_with("X-Other", &signature), body));
        assert!(!blank_secret.verify(header_with("X-Signature", &signature), body));
        assert!(!blank_header.verify(header_with("X-Signature", &signature), body));
        assert!(!spec(SignatureAlgorithm::Sha256).is_presented(headers(&[])));
    }

    #[test]
    fn the_outbound_only_auth_kinds_are_never_presented_by_an_inbound_call() {
        for expected in [
            HttpAuth::Query {
                name: "k".to_owned(),
                value: "v".to_owned(),
            },
            HttpAuth::Digest {
                username: "u".to_owned(),
                password: "p".to_owned(),
            },
        ] {
            assert!(!presents(
                &expected,
                headers(&[("authorization", "Bearer v")])
            ));
        }
    }

    #[test]
    fn an_algorithm_name_outside_the_three_is_not_parsed() {
        assert_eq!(
            SignatureAlgorithm::parse("sha256"),
            Some(SignatureAlgorithm::Sha256)
        );
        assert_eq!(SignatureAlgorithm::parse("md5"), None);
        assert_eq!(SignatureAlgorithm::parse("SHA256"), None);
    }

    fn comparisons_during(work: impl FnOnce()) -> usize {
        COMPARISONS.with(|count| count.set(0));
        work();
        COMPARISONS.with(|count| count.get())
    }

    #[test]
    fn equal_values_are_equal_and_nothing_else_is() {
        assert!(constant_time_equals(b"abc", b"abc"));
        assert!(constant_time_equals(b"", b""));
        assert!(!constant_time_equals(b"abc", b"abd"));
        assert!(!constant_time_equals(b"abc", b"abcd"));
        assert!(!constant_time_equals(b"abcd", b"abc"));
        assert!(!constant_time_equals(b"", b"a"));
    }

    #[test]
    fn a_bearer_token_must_be_presented_as_a_bearer_authorization() {
        let expected = bearer();

        assert!(presents(
            &expected,
            headers(&[("Authorization", "Bearer s3cret")])
        ));
        assert!(presents(
            &expected,
            headers(&[("authorization", "bearer s3cret")])
        ));
        assert!(presents(
            &expected,
            headers(&[("authorization", "BEARER s3cret")])
        ));
    }

    #[test]
    fn a_bearer_token_that_is_wrong_missing_or_in_another_scheme_is_refused() {
        let expected = bearer();

        for value in [
            "Bearer wrong",
            "Bearer s3cre",
            "Bearer s3cret2",
            "Bearer ",
            "Bearer  s3cret",
            "Bearer s3cret ",
            "Bearers3cret",
            "Basic s3cret",
            "Token s3cret",
            "s3cret",
            "",
        ] {
            assert!(
                !presents(&expected, headers(&[("authorization", value)])),
                "`{value}`"
            );
        }
        assert!(!presents(&expected, headers(&[])));
        assert!(!presents(
            &expected,
            headers(&[("x-authorization", "Bearer s3cret")])
        ));
    }

    #[test]
    fn a_basic_credential_is_presented_as_base64_of_user_colon_password() {
        let expected = basic();
        let good = format!("Basic {}", STANDARD.encode("alice:p:ss word"));
        let good: &'static str = Box::leak(good.into_boxed_str());

        assert!(presents(&expected, headers(&[("authorization", good)])));
    }

    #[test]
    fn a_basic_credential_that_is_wrong_or_malformed_is_refused() {
        let expected = basic();
        let encode = |text: &str| -> &'static str {
            Box::leak(format!("Basic {}", STANDARD.encode(text)).into_boxed_str())
        };

        for value in [
            encode("alice:wrong"),
            encode("bob:p:ss word"),
            encode("alice"),
            encode("alice:p:ss word "),
            encode(":p:ss word"),
            "Basic !!!not-base64!!!",
            "Basic ",
            "Bearer YWxpY2U6cDpzcyB3b3Jk",
        ] {
            assert!(
                !presents(&expected, headers(&[("authorization", value)])),
                "`{value}`"
            );
        }
        assert!(!presents(&expected, headers(&[])));
    }

    #[test]
    fn a_header_credential_is_the_named_header_with_exactly_its_value() {
        let expected = header_auth();

        assert!(presents(&expected, headers(&[("X-Api-Key", "k3y")])));
        assert!(presents(&expected, headers(&[("x-api-key", "k3y")])));
        assert!(!presents(&expected, headers(&[("x-api-key", "K3Y")])));
        assert!(!presents(&expected, headers(&[("x-api-key", "k3y ")])));
        assert!(!presents(&expected, headers(&[("x-api-key", "")])));
        assert!(!presents(&expected, headers(&[("x-other-key", "k3y")])));
        assert!(!presents(&expected, headers(&[])));
    }

    #[test]
    fn a_credential_never_authorizes_through_the_wrong_header() {
        assert!(!presents(&bearer(), headers(&[("x-api-key", "s3cret")])));
        assert!(!presents(
            &header_auth(),
            headers(&[("authorization", "Bearer k3y")])
        ));
        assert!(!presents(
            &basic(),
            headers(&[("authorization", "Bearer s3cret")])
        ));
    }

    #[test]
    fn every_secret_is_compared_through_the_constant_time_comparison() {
        let cases: [(HttpAuth, &'static [(&'static str, &'static str)]); 3] = [
            (bearer(), &[("authorization", "Bearer nope")]),
            (basic(), &[("authorization", "Basic YWxpY2U6bm9wZQ==")]),
            (header_auth(), &[("x-api-key", "nope")]),
        ];

        for (expected, pairs) in cases {
            let count = comparisons_during(|| {
                assert!(!presents(&expected, headers(pairs)));
            });

            assert_eq!(count, 1, "{expected:?}");
        }
    }

    #[test]
    fn a_blank_expected_secret_is_never_presented() {
        let blank_bearer = |token: &str| HttpAuth::Bearer {
            token: token.to_owned(),
        };
        for token in ["", " ", "   "] {
            let presented: &'static str = Box::leak(format!("Bearer {token}").into_boxed_str());
            assert!(
                !presents(
                    &blank_bearer(token),
                    headers(&[("authorization", presented)])
                ),
                "`{token}`"
            );
        }

        let encode = |text: &str| -> &'static str {
            Box::leak(format!("Basic {}", STANDARD.encode(text)).into_boxed_str())
        };
        for (username, password) in [("alice", ""), ("", "p4ss"), ("", ""), ("alice", " ")] {
            let expected = HttpAuth::Basic {
                username: username.to_owned(),
                password: password.to_owned(),
            };
            assert!(
                !presents(
                    &expected,
                    headers(&[("authorization", encode(&format!("{username}:{password}")))])
                ),
                "`{username}`:`{password}`"
            );
        }

        for (name, value) in [("X-Api-Key", ""), ("X-Api-Key", "  "), ("", "k3y")] {
            let expected = HttpAuth::Header {
                name: name.to_owned(),
                value: value.to_owned(),
            };
            let pairs: &'static [(&'static str, &'static str)] = Box::leak(Box::new([(
                Box::leak(name.to_owned().into_boxed_str()) as &'static str,
                Box::leak(value.to_owned().into_boxed_str()) as &'static str,
            )]));
            assert!(!presents(&expected, headers(pairs)), "`{name}`:`{value}`");
        }
    }

    #[test]
    fn a_path_fingerprint_is_stable_short_and_does_not_show_the_path() {
        let first = path_fingerprint("paiement/recu");

        assert_eq!(first, path_fingerprint("paiement/recu"));
        assert_eq!(first.len(), 32);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!first.contains("paiement"));
        assert_ne!(first, path_fingerprint("paiement/recu2"));
        assert_ne!(first, path_fingerprint("paiement-recu"));
    }

    #[test]
    fn the_fingerprint_of_a_known_path_never_changes() {
        assert_eq!(
            path_fingerprint("ping"),
            "758d61f26a44448384e5c4468a0dcb7a".to_owned()
        );
    }
}
