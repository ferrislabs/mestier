use aws_lc_rs::digest;
use md5::{Digest, Md5};

fn md5(data: &[u8]) -> Vec<u8> {
    Md5::digest(data).to_vec()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestAlgorithm {
    Md5,
    Md5Session,
    Sha256,
    Sha256Session,
    Sha512Trunc256,
    Sha512Trunc256Session,
}

impl DigestAlgorithm {
    fn parse(token: &str) -> Option<Self> {
        match token.to_ascii_uppercase().as_str() {
            "MD5" => Some(Self::Md5),
            "MD5-SESS" => Some(Self::Md5Session),
            "SHA-256" => Some(Self::Sha256),
            "SHA-256-SESS" => Some(Self::Sha256Session),
            "SHA-512-256" => Some(Self::Sha512Trunc256),
            "SHA-512-256-SESS" => Some(Self::Sha512Trunc256Session),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Md5Session => "MD5-sess",
            Self::Sha256 => "SHA-256",
            Self::Sha256Session => "SHA-256-sess",
            Self::Sha512Trunc256 => "SHA-512-256",
            Self::Sha512Trunc256Session => "SHA-512-256-sess",
        }
    }

    fn is_session(self) -> bool {
        matches!(
            self,
            Self::Md5Session | Self::Sha256Session | Self::Sha512Trunc256Session
        )
    }

    fn hash(self, input: &str) -> String {
        let bytes = match self {
            Self::Md5 | Self::Md5Session => md5(input.as_bytes()),
            Self::Sha256 | Self::Sha256Session => digest::digest(&digest::SHA256, input.as_bytes())
                .as_ref()
                .to_vec(),
            Self::Sha512Trunc256 | Self::Sha512Trunc256Session => {
                digest::digest(&digest::SHA512_256, input.as_bytes())
                    .as_ref()
                    .to_vec()
            }
        };
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    pub realm: String,
    pub nonce: String,
    pub opaque: Option<String>,
    pub algorithm: DigestAlgorithm,
    pub algorithm_declared: bool,
    pub qop_auth: bool,
}

fn parse_parameters(text: &str) -> Vec<(String, String)> {
    let mut parameters = Vec::new();
    let mut characters = text.chars().peekable();
    loop {
        while characters
            .peek()
            .is_some_and(|c| c.is_whitespace() || *c == ',')
        {
            characters.next();
        }
        let mut key = String::new();
        while let Some(c) = characters.peek().copied() {
            if c == '=' || c == ',' {
                break;
            }
            key.push(c);
            characters.next();
        }
        if key.trim().is_empty() && characters.peek().is_none() {
            break;
        }
        let mut value = String::new();
        if characters.peek() == Some(&'=') {
            characters.next();
            if characters.peek() == Some(&'"') {
                characters.next();
                while let Some(c) = characters.next() {
                    match c {
                        '\\' => {
                            if let Some(escaped) = characters.next() {
                                value.push(escaped);
                            }
                        }
                        '"' => break,
                        other => value.push(other),
                    }
                }
            } else {
                while let Some(c) = characters.peek().copied() {
                    if c == ',' {
                        break;
                    }
                    value.push(c);
                    characters.next();
                }
            }
        }
        parameters.push((key.trim().to_ascii_lowercase(), value.trim().to_owned()));
    }
    parameters
}

pub fn parse_challenge(header: &str) -> Option<Challenge> {
    let header = header.trim_start();
    let (scheme, rest) = header.split_once(char::is_whitespace)?;
    if !scheme.eq_ignore_ascii_case("digest") {
        return None;
    }
    let parameters = parse_parameters(rest);
    let find = |name: &str| {
        parameters
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };

    let realm = find("realm")?;
    let nonce = find("nonce")?;
    let declared = find("algorithm");
    let algorithm = match &declared {
        Some(token) => DigestAlgorithm::parse(token)?,
        None => DigestAlgorithm::Md5,
    };
    let qop_auth = match find("qop") {
        Some(offered) => {
            if !offered
                .split(',')
                .any(|option| option.trim().eq_ignore_ascii_case("auth"))
            {
                return None;
            }
            true
        }
        None => false,
    };

    Some(Challenge {
        realm,
        nonce,
        opaque: find("opaque"),
        algorithm,
        algorithm_declared: declared.is_some(),
        qop_auth,
    })
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

pub fn authorization(
    challenge: &Challenge,
    username: &str,
    password: &str,
    method: &str,
    uri: &str,
    nonce_count: u32,
    client_nonce: &str,
) -> String {
    let algorithm = challenge.algorithm;
    let mut first = algorithm.hash(&format!("{username}:{}:{password}", challenge.realm));
    if algorithm.is_session() {
        first = algorithm.hash(&format!("{first}:{}:{client_nonce}", challenge.nonce));
    }
    let second = algorithm.hash(&format!("{method}:{uri}"));
    let count = format!("{nonce_count:08x}");
    let response = if challenge.qop_auth {
        algorithm.hash(&format!(
            "{first}:{}:{count}:{client_nonce}:auth:{second}",
            challenge.nonce
        ))
    } else {
        algorithm.hash(&format!("{first}:{}:{second}", challenge.nonce))
    };

    let mut header = format!(
        "Digest username={}, realm={}, nonce={}, uri={}",
        quoted(username),
        quoted(&challenge.realm),
        quoted(&challenge.nonce),
        quoted(uri),
    );
    if challenge.algorithm_declared {
        header.push_str(&format!(", algorithm={}", algorithm.name()));
    }
    header.push_str(&format!(", response={}", quoted(&response)));
    if challenge.qop_auth {
        header.push_str(&format!(
            ", qop=auth, nc={count}, cnonce={}",
            quoted(client_nonce)
        ));
    }
    if let Some(opaque) = &challenge.opaque {
        header.push_str(&format!(", opaque={}", quoted(opaque)));
    }
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn md5_matches_the_rfc_1321_test_suite() {
        for (input, expected) in [
            ("", "d41d8cd98f00b204e9800998ecf8427e"),
            ("a", "0cc175b9c0f1b6a831c399e269772661"),
            ("abc", "900150983cd24fb0d6963f7d28e17f72"),
            ("message digest", "f96b697d7cb7938d525a2f31aaf161d0"),
            (
                "abcdefghijklmnopqrstuvwxyz",
                "c3fcd3d76192e4007dfb496cca67e13b",
            ),
            (
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                "d174ab98d277d9f5a5611c2c9f419d9f",
            ),
            (
                "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
                "57edf4a22be3c955ac49da2e2107b67a",
            ),
        ] {
            assert_eq!(hex(&md5(input.as_bytes())), expected, "{input}");
        }
        assert_eq!(hex(&md5(&[b'a'; 1000])), "cabe45dcc9ae5b66ba86600cca6b8ba8");
    }

    const RFC_2617_CHALLENGE: &str = r#"Digest realm="testrealm@host.com", qop="auth,auth-int", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093", opaque="5ccc069c403ebaf9f0171e9517f40e41""#;

    fn rfc_7616_challenge(algorithm: &str) -> String {
        format!(
            r#"Digest realm="http-auth@example.org", qop="auth, auth-int", algorithm={algorithm}, nonce="7ypf/xlj9XXwfDPEoM4URrv/xwf94BcCAzFZH4GiTo0v", opaque="FQhe/qaU925kfnzjCev0ciny7QMkPqMAFRtzCUYo5tdS""#
        )
    }

    const RFC_7616_CNONCE: &str = "f2/wE4q74E6zIJEtWaHKaf5wv/H5QzzpXusqGemxURZJ";

    #[test]
    fn the_rfc_2617_example_is_answered_with_the_published_response() {
        let challenge = parse_challenge(RFC_2617_CHALLENGE).unwrap();

        let header = authorization(
            &challenge,
            "Mufasa",
            "Circle Of Life",
            "GET",
            "/dir/index.html",
            1,
            "0a4f113b",
        );

        assert!(
            header.contains(r#"response="6629fae49393a05397450978507c4ef1""#),
            "{header}"
        );
        assert!(header.contains("qop=auth, nc=00000001"), "{header}");
        assert!(header.contains(r#"opaque="5ccc069c403ebaf9f0171e9517f40e41""#));
        assert!(!header.contains("algorithm="), "{header}");
    }

    #[test]
    fn the_rfc_7616_examples_are_answered_with_the_published_responses() {
        for (algorithm, expected) in [
            ("MD5", "8ca523f5e9506fed4657c9700eebdbec"),
            (
                "SHA-256",
                "753927fa0e85d155564e2e272a28d1802ca10daf4496794697cf8db5856cb6c1",
            ),
            (
                "SHA-512-256",
                "430d05014cecc49cab6fbe03176d41a1da86cbfe24a16580e22aaad928d960d0",
            ),
        ] {
            let challenge = parse_challenge(&rfc_7616_challenge(algorithm)).unwrap();

            let header = authorization(
                &challenge,
                "Mufasa",
                "Circle of Life",
                "GET",
                "/dir/index.html",
                1,
                RFC_7616_CNONCE,
            );

            assert!(
                header.contains(&format!(r#"response="{expected}""#)),
                "{algorithm}: {header}"
            );
            assert!(
                header.contains(&format!("algorithm={algorithm}")),
                "{header}"
            );
        }
    }

    #[test]
    fn a_challenge_without_qop_uses_the_older_response_formula() {
        let challenge = parse_challenge(r#"Digest realm="r", nonce="n", algorithm=MD5"#).unwrap();

        let header = authorization(&challenge, "u", "p", "GET", "/", 1, "ignored");

        assert!(
            header.contains(r#"response="f23121ca97143c6b358da620220c83b4""#),
            "{header}"
        );
        assert!(!header.contains("qop="), "{header}");
        assert!(!header.contains("cnonce"), "{header}");
    }

    #[test]
    fn a_challenge_needs_a_realm_and_a_nonce_and_a_usable_scheme() {
        assert_eq!(parse_challenge(r#"Digest realm="r""#), None);
        assert_eq!(parse_challenge(r#"Digest nonce="n""#), None);
        assert_eq!(parse_challenge(r#"Basic realm="r""#), None);
        assert_eq!(parse_challenge(""), None);
        assert_eq!(
            parse_challenge(r#"Digest realm="r", nonce="n", algorithm=SHA-3"#),
            None
        );
        assert_eq!(
            parse_challenge(r#"Digest realm="r", nonce="n", qop="auth-int""#),
            None,
            "auth-int alone cannot be answered"
        );
    }

    #[test]
    fn quoted_values_keep_commas_and_escaped_quotes() {
        let challenge =
            parse_challenge(r#"Digest realm="a, b \"c\"", nonce="n,1", qop=auth"#).unwrap();

        assert_eq!(challenge.realm, r#"a, b "c""#);
        assert_eq!(challenge.nonce, "n,1");
        assert!(challenge.qop_auth);
    }

    #[test]
    fn the_session_variants_fold_the_client_nonce_into_the_first_hash() {
        let plain =
            parse_challenge(r#"Digest realm="r", nonce="n", qop=auth, algorithm=MD5"#).unwrap();
        let session =
            parse_challenge(r#"Digest realm="r", nonce="n", qop=auth, algorithm=MD5-sess"#)
                .unwrap();

        let a = authorization(&plain, "u", "p", "GET", "/", 1, "c");
        let b = authorization(&session, "u", "p", "GET", "/", 1, "c");

        assert_ne!(a, b);
        assert!(b.contains("algorithm=MD5-sess"), "{b}");
    }

    #[test]
    fn the_header_escapes_quotes_in_the_user_name() {
        let challenge = parse_challenge(r#"Digest realm="r", nonce="n""#).unwrap();

        let header = authorization(&challenge, r#"a"b"#, "p", "GET", "/", 1, "c");

        assert!(header.contains(r#"username="a\"b""#), "{header}");
    }
}
