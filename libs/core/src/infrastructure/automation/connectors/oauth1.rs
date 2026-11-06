use aws_lc_rs::hmac;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::Url;

pub struct Signer<'a> {
    pub consumer_key: &'a str,
    pub consumer_secret: &'a str,
    pub token: Option<&'a str>,
    pub token_secret: Option<&'a str>,
    pub signature_method: &'a str,
}

pub fn percent_encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn base_url(url: &Url) -> Result<String, String> {
    let host = url
        .host_str()
        .ok_or_else(|| "the url has no host to sign".to_owned())?
        .to_ascii_lowercase();
    let default = matches!(
        (url.scheme(), url.port()),
        ("http", None | Some(80)) | ("https", None | Some(443))
    );
    let authority = match url.port() {
        Some(port) if !default => format!("{host}:{port}"),
        _ => host,
    };
    Ok(format!("{}://{authority}{}", url.scheme(), url.path()))
}

pub fn authorization_header(
    signer: &Signer<'_>,
    http_method: &str,
    url: &Url,
    extra: &[(String, String)],
    nonce: &str,
    timestamp: u64,
) -> Result<String, String> {
    if signer.signature_method == "PLAINTEXT" && url.scheme() != "https" {
        return Err("a PLAINTEXT signature is only sent over https".to_owned());
    }
    let mut oauth: Vec<(String, String)> = vec![
        (
            "oauth_consumer_key".to_owned(),
            signer.consumer_key.to_owned(),
        ),
        ("oauth_nonce".to_owned(), nonce.to_owned()),
        (
            "oauth_signature_method".to_owned(),
            signer.signature_method.to_owned(),
        ),
        ("oauth_timestamp".to_owned(), timestamp.to_string()),
        ("oauth_version".to_owned(), "1.0".to_owned()),
    ];
    if let Some(token) = signer.token {
        oauth.push(("oauth_token".to_owned(), token.to_owned()));
    }
    for (name, value) in extra {
        if name.starts_with("oauth_") {
            oauth.push((name.clone(), value.clone()));
        }
    }

    let mut signed: Vec<(String, String)> = oauth
        .iter()
        .map(|(name, value)| (percent_encode(name), percent_encode(value)))
        .collect();
    for (name, value) in url.query_pairs() {
        signed.push((percent_encode(&name), percent_encode(&value)));
    }
    for (name, value) in extra {
        if !name.starts_with("oauth_") {
            signed.push((percent_encode(name), percent_encode(value)));
        }
    }
    signed.sort();
    let normalized = signed
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&");

    let key = format!(
        "{}&{}",
        percent_encode(signer.consumer_secret),
        percent_encode(signer.token_secret.unwrap_or(""))
    );
    let signature = match signer.signature_method {
        "PLAINTEXT" => key,
        method => {
            let base = format!(
                "{}&{}&{}",
                http_method.to_ascii_uppercase(),
                percent_encode(&base_url(url)?),
                percent_encode(&normalized)
            );
            let algorithm = match method {
                "HMAC-SHA1" => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
                "HMAC-SHA256" => hmac::HMAC_SHA256,
                other => return Err(format!("unsupported OAuth1 signature method `{other}`")),
            };
            let tag = hmac::sign(&hmac::Key::new(algorithm, key.as_bytes()), base.as_bytes());
            STANDARD.encode(tag.as_ref())
        }
    };

    oauth.push(("oauth_signature".to_owned(), signature));
    oauth.sort();
    let fields = oauth
        .iter()
        .map(|(name, value)| format!("{}=\"{}\"", percent_encode(name), percent_encode(value)))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!("OAuth {fields}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn twitter_signer<'a>(method: &'a str) -> Signer<'a> {
        Signer {
            consumer_key: "xvz1evFS4wEEPTGEFPHBog",
            consumer_secret: "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw",
            token: Some("370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb"),
            token_secret: Some("LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE"),
            signature_method: method,
        }
    }

    #[test]
    fn it_reproduces_the_published_signature_of_a_signed_request() {
        let url =
            Url::parse("https://api.twitter.com/1.1/statuses/update.json?include_entities=true")
                .unwrap();
        let extra = vec![(
            "status".to_owned(),
            "Hello Ladies + Gentlemen, a signed OAuth request!".to_owned(),
        )];

        let header = authorization_header(
            &twitter_signer("HMAC-SHA1"),
            "post",
            &url,
            &extra,
            "kYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg",
            1318622958,
        )
        .unwrap();

        assert!(
            header.contains(r#"oauth_signature="hCtSmYh%2BiHYCEqBWrE7C7hYmtUk%3D""#),
            "{header}"
        );
        assert!(header.starts_with("OAuth "), "{header}");
        assert!(header.contains(r#"oauth_version="1.0""#), "{header}");
        assert!(!header.contains("status"), "{header}");
        assert!(!header.contains("include_entities"), "{header}");
    }

    #[test]
    fn the_percent_encoding_keeps_only_the_unreserved_characters() {
        assert_eq!(
            percent_encode("Ladies + Gentlemen"),
            "Ladies%20%2B%20Gentlemen"
        );
        assert_eq!(percent_encode("a-b._c~d"), "a-b._c~d");
        assert_eq!(percent_encode("é/="), "%C3%A9%2F%3D");
    }

    #[test]
    fn plaintext_signs_with_the_two_secrets_joined() {
        let url = Url::parse("https://api.example.com/x").unwrap();

        let header =
            authorization_header(&twitter_signer("PLAINTEXT"), "GET", &url, &[], "n", 1).unwrap();

        let expected = format!(
            "{}%26{}",
            percent_encode("kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw"),
            percent_encode("LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE"),
        );
        assert!(
            header.contains(&format!(r#"oauth_signature="{expected}""#)),
            "{header}"
        );
    }

    #[test]
    fn plaintext_is_never_sent_over_plain_http() {
        let url = Url::parse("http://api.example.com/x").unwrap();

        let refused = authorization_header(&twitter_signer("PLAINTEXT"), "GET", &url, &[], "n", 1);

        assert!(refused.is_err());
        assert!(
            authorization_header(&twitter_signer("HMAC-SHA1"), "GET", &url, &[], "n", 1).is_ok()
        );
    }

    #[test]
    fn a_request_without_a_token_signs_with_an_empty_token_secret() {
        let url = Url::parse("https://api.example.com/x").unwrap();
        let signer = Signer {
            consumer_key: "key",
            consumer_secret: "secret",
            token: None,
            token_secret: None,
            signature_method: "HMAC-SHA256",
        };

        let header = authorization_header(&signer, "GET", &url, &[], "n", 1).unwrap();

        assert!(!header.contains("oauth_token"), "{header}");
        assert!(
            header.contains(r#"oauth_signature_method="HMAC-SHA256""#),
            "{header}"
        );
    }

    #[test]
    fn an_unknown_signature_method_is_refused() {
        let url = Url::parse("https://api.example.com/x").unwrap();

        let error = authorization_header(&twitter_signer("RSA-SHA1"), "GET", &url, &[], "n", 1)
            .unwrap_err();

        assert!(error.contains("RSA-SHA1"), "{error}");
    }

    #[test]
    fn the_base_url_drops_the_default_port_the_query_and_the_case_of_the_host() {
        let url = Url::parse("HTTPS://API.Example.com:443/a/b?x=1#frag").unwrap();
        assert_eq!(base_url(&url).unwrap(), "https://api.example.com/a/b");

        let url = Url::parse("http://api.example.com:8080/a").unwrap();
        assert_eq!(base_url(&url).unwrap(), "http://api.example.com:8080/a");
    }

    #[test]
    fn extra_oauth_parameters_travel_in_the_header_and_other_parameters_do_not() {
        let url = Url::parse("https://api.example.com/request_token").unwrap();
        let extra = vec![
            (
                "oauth_callback".to_owned(),
                "https://app.example.com/cb".to_owned(),
            ),
            ("other".to_owned(), "value".to_owned()),
        ];

        let header =
            authorization_header(&twitter_signer("HMAC-SHA1"), "POST", &url, &extra, "n", 1)
                .unwrap();

        assert!(
            header.contains(r#"oauth_callback="https%3A%2F%2Fapp.example.com%2Fcb""#),
            "{header}"
        );
        assert!(!header.contains("other"), "{header}");
    }
}
