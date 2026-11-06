use std::time::Duration;

use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Utc;
use reqwest::{
    Certificate, Client, Identity, Method, RequestBuilder, Response, StatusCode, Url,
    header::{AUTHORIZATION, WWW_AUTHENTICATE},
    redirect::Policy,
};
use serde_json::{Map, Value};

use super::{digest_auth, jwt_auth, oauth1};
use crate::infrastructure::automation::webhook::{
    address_policy::PrivateNetworkAccess, resolver::GuardedResolver,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CustomAuth {
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OAuth1Auth {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub token: String,
    pub token_secret: String,
    pub signature_method: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientCertificate {
    pub certificate: String,
    pub private_key: String,
    pub passphrase: Option<String>,
    pub ca_certificate: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpAuth {
    Bearer { token: String },
    Basic { username: String, password: String },
    Header { name: String, value: String },
    Query { name: String, value: String },
    Digest { username: String, password: String },
    Jwt { algorithm: String, key: String },
    Custom(CustomAuth),
    OAuth1(OAuth1Auth),
    ClientCertificate(ClientCertificate),
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AuthFromCredentialError {
    #[error("unsupported auth credential kind `{kind}`")]
    UnsupportedKind { kind: String },
    #[error("credential is missing field `{field}`")]
    MissingField { field: String },
    #[error("the credential's custom definition is not usable")]
    UnusableDefinition,
}

impl AuthFromCredentialError {
    #[cfg(test)]
    pub(crate) fn missing(field: &str) -> Self {
        Self::MissingField {
            field: field.to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreparedRequest {
    pub method: Method,
    pub url: Url,
    pub headers: Vec<(String, String)>,
    pub body: Option<Value>,
}

impl PreparedRequest {
    pub fn new(
        method: Method,
        url: Url,
        headers: Vec<(String, String)>,
        body: Option<Value>,
    ) -> Self {
        Self {
            method,
            url,
            headers,
            body,
        }
    }

    pub fn set_header(&mut self, name: &str, value: String) {
        self.headers
            .retain(|(existing, _)| !existing.eq_ignore_ascii_case(name));
        self.headers.push((name.to_owned(), value));
    }

    pub fn set_query(&mut self, name: &str, value: &str) {
        let mut kept: Vec<(String, String)> = self
            .url
            .query_pairs()
            .filter(|(existing, _)| existing != name)
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        kept.push((name.to_owned(), value.to_owned()));
        self.url.set_query(None);
        let mut pairs = self.url.query_pairs_mut();
        for (key, value) in &kept {
            pairs.append_pair(key, value);
        }
    }

    pub fn body_bytes(&self) -> Option<Vec<u8>> {
        self.body.as_ref().map(|value| {
            serde_json::to_vec(value)
                .expect("a serde_json::Value serializes into a JSON body without failing")
        })
    }

    fn has_header(&self, name: &str) -> bool {
        self.headers
            .iter()
            .any(|(existing, _)| existing.eq_ignore_ascii_case(name))
    }

    fn builder(&self, client: &Client, timeout: Duration) -> RequestBuilder {
        let mut request = client
            .request(self.method.clone(), self.url.clone())
            .timeout(timeout);
        for (name, value) in &self.headers {
            request = request.header(name, value);
        }
        if let Some(bytes) = self.body_bytes() {
            if !self.has_header("content-type") {
                request = request.header("content-type", "application/json");
            }
            request = request.body(bytes);
        }
        request
    }

    fn request_uri(&self) -> String {
        match self.url.query() {
            Some(query) => format!("{}?{query}", self.url.path()),
            None => self.url.path().to_owned(),
        }
    }
}

fn random_hex(length: usize) -> String {
    let mut bytes = vec![0u8; length];
    SystemRandom::new()
        .fill(&mut bytes)
        .expect("the system random source answers");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn scalar_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn text_pairs(section: Option<&Value>) -> Vec<(String, String)> {
    section
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .map(|(name, value)| (name.clone(), scalar_text(value)))
                .collect()
        })
        .unwrap_or_default()
}

impl HttpAuth {
    pub fn prepare(&self, request: &mut PreparedRequest) -> Result<(), String> {
        match self {
            HttpAuth::Bearer { token } => {
                request.set_header("authorization", format!("Bearer {token}"));
            }
            HttpAuth::Basic { username, password } => {
                let encoded = STANDARD.encode(format!("{username}:{password}"));
                request.set_header("authorization", format!("Basic {encoded}"));
            }
            HttpAuth::Header { name, value } => request.set_header(name, value.clone()),
            HttpAuth::Query { name, value } => request.set_query(name, value),
            HttpAuth::Digest { .. } | HttpAuth::ClientCertificate(_) => {}
            HttpAuth::Jwt { algorithm, key } => {
                let issued_at = Utc::now().timestamp().max(0) as u64;
                let token = jwt_auth::sign(algorithm, key, issued_at)?;
                request.set_header("authorization", format!("Bearer {token}"));
            }
            HttpAuth::Custom(custom) => {
                for (name, value) in &custom.headers {
                    request.set_header(name, value.clone());
                }
                for (name, value) in &custom.query {
                    request.set_query(name, value);
                }
                if !custom.body.is_empty() {
                    match request.body.as_mut() {
                        None => request.body = Some(Value::Object(custom.body.clone())),
                        Some(Value::Object(existing)) => {
                            for (name, value) in &custom.body {
                                existing.insert(name.clone(), value.clone());
                            }
                        }
                        Some(_) => {
                            return Err(
                                "the custom authentication adds body fields, but the request body is not an object"
                                    .to_owned(),
                            );
                        }
                    }
                }
            }
            HttpAuth::OAuth1(auth) => {
                let signer = oauth1::Signer {
                    consumer_key: &auth.consumer_key,
                    consumer_secret: &auth.consumer_secret,
                    token: Some(&auth.token),
                    token_secret: Some(&auth.token_secret),
                    signature_method: &auth.signature_method,
                };
                let header = oauth1::authorization_header(
                    &signer,
                    request.method.as_str(),
                    &request.url,
                    &[],
                    &random_hex(16),
                    Utc::now().timestamp().max(0) as u64,
                )?;
                request.set_header("authorization", header);
            }
        }
        Ok(())
    }
}

pub fn auth_from_credential(
    kind: &str,
    fields: &Map<String, Value>,
) -> Result<HttpAuth, AuthFromCredentialError> {
    let field = |name: &'static str| -> Result<String, AuthFromCredentialError> {
        fields
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(AuthFromCredentialError::MissingField {
                field: name.to_owned(),
            })
    };
    let optional = |name: &'static str| -> Option<String> {
        fields
            .get(name)
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned)
    };

    match kind {
        "bearer_token" => Ok(HttpAuth::Bearer {
            token: field("token")?,
        }),
        "http_basic" => Ok(HttpAuth::Basic {
            username: field("username")?,
            password: field("password")?,
        }),
        "http_header" => Ok(HttpAuth::Header {
            name: field("header_name")?,
            value: field("header_value")?,
        }),
        "api_key_query" => Ok(HttpAuth::Query {
            name: field("param_name")?,
            value: field("param_value")?,
        }),
        "digest" => Ok(HttpAuth::Digest {
            username: field("username")?,
            password: field("password")?,
        }),
        "jwt" => Ok(HttpAuth::Jwt {
            algorithm: optional("algorithm").unwrap_or_else(|| "HS256".to_owned()),
            key: field("key")?,
        }),
        "custom_auth" => {
            let definition = fields.get("json").and_then(Value::as_object).ok_or(
                AuthFromCredentialError::MissingField {
                    field: "json".to_owned(),
                },
            )?;
            let body = match definition.get("body") {
                None => Map::new(),
                Some(Value::Object(body)) => body.clone(),
                Some(_) => return Err(AuthFromCredentialError::UnusableDefinition),
            };
            Ok(HttpAuth::Custom(CustomAuth {
                headers: text_pairs(definition.get("headers")),
                query: text_pairs(definition.get("qs")),
                body,
            }))
        }
        "oauth1" => Ok(HttpAuth::OAuth1(OAuth1Auth {
            consumer_key: field("consumer_key")?,
            consumer_secret: field("consumer_secret")?,
            token: field("oauth_token")?,
            token_secret: field("oauth_token_secret")?,
            signature_method: optional("signature_method")
                .unwrap_or_else(|| "HMAC-SHA1".to_owned()),
        })),
        "mtls" => Ok(HttpAuth::ClientCertificate(ClientCertificate {
            certificate: field("certificate")?,
            private_key: field("private_key")?,
            passphrase: optional("passphrase"),
            ca_certificate: optional("ca_certificate"),
        })),
        other => Err(AuthFromCredentialError::UnsupportedKind {
            kind: other.to_owned(),
        }),
    }
}

pub fn client_for_auth(
    base: &Client,
    access: Option<PrivateNetworkAccess>,
    auth: Option<&HttpAuth>,
) -> Result<Client, String> {
    let Some(HttpAuth::ClientCertificate(certificate)) = auth else {
        return Ok(base.clone());
    };
    if certificate.private_key.contains("ENCRYPTED") {
        return Err(
            "an encrypted client private key is not supported: store it unencrypted".to_owned(),
        );
    }
    let identity = Identity::from_pkcs8_pem(
        certificate.certificate.as_bytes(),
        certificate.private_key.as_bytes(),
    )
    .map_err(|error| format!("the client certificate or its private key is not usable: {error}"))?;

    let mut builder = Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .identity(identity);
    if let Some(access) = access {
        builder = builder.dns_resolver(GuardedResolver::new(access));
    }
    if let Some(authority) = &certificate.ca_certificate {
        let authority = Certificate::from_pem(authority.as_bytes())
            .map_err(|error| format!("the certificate authority is not usable: {error}"))?;
        builder = builder.add_root_certificate(authority);
    }
    builder
        .build()
        .map_err(|error| format!("building the client with its certificate failed: {error}"))
}

pub async fn send(
    client: &Client,
    request: &PreparedRequest,
    timeout: Duration,
    auth: Option<&HttpAuth>,
) -> Result<Response, reqwest::Error> {
    let first = request.builder(client, timeout).send().await?;
    let Some(HttpAuth::Digest { username, password }) = auth else {
        return Ok(first);
    };
    if first.status() != StatusCode::UNAUTHORIZED {
        return Ok(first);
    }
    let Some(challenge) = first
        .headers()
        .get_all(WWW_AUTHENTICATE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(digest_auth::parse_challenge)
    else {
        return Ok(first);
    };

    let header = digest_auth::authorization(
        &challenge,
        username,
        password,
        request.method.as_str(),
        &request.request_uri(),
        1,
        &random_hex(16),
    );
    drop(first);
    request
        .builder(client, timeout)
        .header(AUTHORIZATION, header)
        .send()
        .await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::infrastructure::automation::connectors::test_support::{StubResponse, TestServer};

    const TEST_CERTIFICATE: &str = r#"-----BEGIN CERTIFICATE-----
MIIDETCCAfmgAwIBAgIUPmaRcFivocaYn9DoKkjlPbCebB0wDQYJKoZIhvcNAQEL
BQAwFzEVMBMGA1UEAwwMbWVzdGllci10ZXN0MCAXDTI2MTEwNjEzMDcxNVoYDzIx
MjYxMDEzMTMwNzE1WjAXMRUwEwYDVQQDDAxtZXN0aWVyLXRlc3QwggEiMA0GCSqG
SIb3DQEBAQUAA4IBDwAwggEKAoIBAQC1Re7yyu+pEph7j3IR1JEA/m3Mf10v+1qT
ZxrkqR8JeMB126cLZGby8U3erM2QxcX781J3uiLyNvElV1s+hDwbBn8rQKy0Iwf6
7ISX43WAOBJS3YL5cm/DD87ryBoFpk95sOVvbgvc+dMh7sot/mibJCP5ULIGIQID
D+GiCxwxCZPEYYj3Brs1TYMlzts7dCfPAeWWNCZ3/wKri1Lu2U8G2S9eOOeAsrmZ
veqwUBRkoVIRejVbD8JI1RUbb4CopL82fRF5cCuGc03MWWUqY8/EEVjQmrL/X3gS
UWx8SShJMZ7G5pnnO7HOX6jfYPJbqpNJXOdtZOytn2mFnjj218kHAgMBAAGjUzBR
MB0GA1UdDgQWBBR3AfGO2X0H6cvQ8YLHYoabRvGcpTAfBgNVHSMEGDAWgBR3AfGO
2X0H6cvQ8YLHYoabRvGcpTAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUA
A4IBAQBGzuXdY46ywWb1OS3pAuvu4mOTDt+qMfPbErVzilP1J+fYGcYyZWN5gJ6w
vnOp5wJkwDR7IF9u6AqyNwA0nPSo59pIWjVTYWpatbnzMKOfqsKvkax4kExAnW0+
ptt7Ifm1nNjvBzVc8bFZ4qPPFMZLc9uw7TFCBjkFHTlsoz8C8E2PDjetz5x/cLju
ulncYz64YCMXNDtHSQXRMFbKBzNAQLwzZ6QjwmYMj4DG29jLEjwzdzFVa2IuNBk8
7fw56RuXvi3UIEKjdcxvHvL0Txxdu1ZW0V+5tsLztMLev0s3W8EJxu2BtK52guT4
R6kWBg7msfrlbZMAJjhIwljCscfp
-----END CERTIFICATE-----"#;
    const TEST_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC1Re7yyu+pEph7
j3IR1JEA/m3Mf10v+1qTZxrkqR8JeMB126cLZGby8U3erM2QxcX781J3uiLyNvEl
V1s+hDwbBn8rQKy0Iwf67ISX43WAOBJS3YL5cm/DD87ryBoFpk95sOVvbgvc+dMh
7sot/mibJCP5ULIGIQIDD+GiCxwxCZPEYYj3Brs1TYMlzts7dCfPAeWWNCZ3/wKr
i1Lu2U8G2S9eOOeAsrmZveqwUBRkoVIRejVbD8JI1RUbb4CopL82fRF5cCuGc03M
WWUqY8/EEVjQmrL/X3gSUWx8SShJMZ7G5pnnO7HOX6jfYPJbqpNJXOdtZOytn2mF
njj218kHAgMBAAECggEAUvHGddL02YyYOAV5gP4eQXGqBQYMOk3NcqAddp6KBJf7
nJIeu8A7uRM/+dFhGErenjwGsqD5p5ILQnZX2mHAbO9Klhx4Sc6zspWx0Bw0ojBF
9iPEb+Z0jDE6EboUahxgxkJyoUud4wVvHdo5KdV8wOxHB6fsZZCJ7qPALxjdU/QT
abpIuCitg+2fbF00ynEpqhQMTv6RcGMs0nYoplAAGDYg+ONq+tmds4+rXgYH+DES
ijz9fnZB21jMftccOQFAQztfZw2Uo6RxKRkSJoroyd+8IqI11XXt0vN6LUpIGDne
dObXB/BRztO/fH5OTAbU61YaA+005dSn+FGxOHY19QKBgQDsb9UZoyx/TzeiDCYt
p6cpsyB22GtdqLD+SOfCdj6Wty4gJS5bhZjADybwd3XNB5tuDAQiWszlgvLnlDSd
LXSQsTXD5jNdTXydSuwXQRgQDUapOTjInShRKah9AFxBzAIZf6p/PSGVF14RAtgo
Wnjx1j+956jMDw39EaBd3VLY7QKBgQDERaIInc71o6Zz6vPUy2f7tCa7VGbXW2X+
1uT7SPEPAGNyOZjXP5M2AstnMIexZNeqbccKViPhGa0QCmcYqkCXc7MtpL8wFmfV
t7CSqpz+8/D4MccZbdLIBbZve+MoDniT6H596o6KWleR4AK4ENl0HgoTMaUFAasI
ILVnJnuvQwKBgQCut3FE8bSXG0XkJJAt2p8HjIvn5sNoQZkRP3Sr2c8kdpbKQE+C
aDXZbxj6R3ywlvd/mbeQX4Lb8ylmgbljkMx7g8hUTA8W2oTEgzH1eS+75OObd2KD
tMavO1Z7kMGcCZJfn1yxi83+K3dioLR1FW4cX5NY1RCCQnm20cistDldfQKBgETw
HFCgdS/sMivm64vjgftztGL560FbNgEDo9vrK9hUD5nwRxHbn65J3v24mwCg5etT
saScyu9moFXRAZM1QYiwt8+hXRHb3M7BPUDWslwOfKYAfekdnPpqn0CbKBdKtell
sQAzIavQvY9cOjhCms5hOrKxLkAK7PLm4jB/4NOXAoGBALnN4kqJgRwzGV+DM3K2
l0LG39Er3GHV8cl7DAIf5S4cRn2F2dludAKAtrdYmWmTocoHCk9df+gDaoqFKSG7
AmGRM/bH+wN7w4Tn0VYkLbQMksp5srMYQ5yjYu4O5xW4Ax8KZ8eL1ltwOO+oqPWQ
td7H8fmaJgKruJIBRIL8AlKR
-----END PRIVATE KEY-----"#;

    fn fields(value: Value) -> Map<String, Value> {
        value.as_object().cloned().unwrap()
    }

    fn request(url: &str) -> PreparedRequest {
        PreparedRequest::new(Method::GET, Url::parse(url).unwrap(), Vec::new(), None)
    }

    fn header<'a>(request: &'a PreparedRequest, name: &str) -> Option<&'a str> {
        request
            .headers
            .iter()
            .find(|(existing, _)| existing.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    fn auth_of(kind: &str, value: Value) -> HttpAuth {
        auth_from_credential(kind, &fields(value)).unwrap()
    }

    #[test]
    fn each_http_kind_becomes_the_auth_of_its_own_scheme() {
        assert_eq!(
            auth_of("bearer_token", json!({ "token": "t" })),
            HttpAuth::Bearer {
                token: "t".to_owned()
            }
        );
        assert_eq!(
            auth_of("http_basic", json!({ "username": "u", "password": "p" })),
            HttpAuth::Basic {
                username: "u".to_owned(),
                password: "p".to_owned()
            }
        );
        assert_eq!(
            auth_of(
                "http_header",
                json!({ "header_name": "X-K", "header_value": "v" })
            ),
            HttpAuth::Header {
                name: "X-K".to_owned(),
                value: "v".to_owned()
            }
        );
        assert_eq!(
            auth_of(
                "api_key_query",
                json!({ "param_name": "key", "param_value": "v" })
            ),
            HttpAuth::Query {
                name: "key".to_owned(),
                value: "v".to_owned()
            }
        );
        assert_eq!(
            auth_of("digest", json!({ "username": "u", "password": "p" })),
            HttpAuth::Digest {
                username: "u".to_owned(),
                password: "p".to_owned()
            }
        );
        assert_eq!(
            auth_of("jwt", json!({ "algorithm": "HS512", "key": "k" })),
            HttpAuth::Jwt {
                algorithm: "HS512".to_owned(),
                key: "k".to_owned()
            }
        );
        assert_eq!(
            auth_of("jwt", json!({ "key": "k" })),
            HttpAuth::Jwt {
                algorithm: "HS256".to_owned(),
                key: "k".to_owned()
            }
        );
        assert_eq!(
            auth_of(
                "custom_auth",
                json!({ "json": { "headers": { "X-T": "a", "X-N": 3 }, "qs": { "q": true }, "body": { "b": 1 } } })
            ),
            HttpAuth::Custom(CustomAuth {
                headers: vec![
                    ("X-N".to_owned(), "3".to_owned()),
                    ("X-T".to_owned(), "a".to_owned())
                ],
                query: vec![("q".to_owned(), "true".to_owned())],
                body: fields(json!({ "b": 1 })),
            })
        );
        assert_eq!(
            auth_of(
                "mtls",
                json!({ "certificate": "C", "private_key": "K", "passphrase": "  ", "ca_certificate": "A" })
            ),
            HttpAuth::ClientCertificate(ClientCertificate {
                certificate: "C".to_owned(),
                private_key: "K".to_owned(),
                passphrase: None,
                ca_certificate: Some("A".to_owned()),
            })
        );
        assert_eq!(
            auth_of(
                "oauth1",
                json!({
                    "consumer_key": "ck", "consumer_secret": "cs",
                    "oauth_token": "t", "oauth_token_secret": "ts",
                })
            ),
            HttpAuth::OAuth1(OAuth1Auth {
                consumer_key: "ck".to_owned(),
                consumer_secret: "cs".to_owned(),
                token: "t".to_owned(),
                token_secret: "ts".to_owned(),
                signature_method: "HMAC-SHA1".to_owned(),
            })
        );
    }

    #[test]
    fn a_credential_kind_no_http_call_consumes_is_unsupported() {
        for kind in ["hmac", "smtp", "imap", "ssh", "ftp", "sql", "odoo_api"] {
            assert_eq!(
                auth_from_credential(kind, &Map::new()),
                Err(AuthFromCredentialError::UnsupportedKind {
                    kind: kind.to_owned()
                }),
                "{kind}"
            );
        }
    }

    #[test]
    fn an_oauth1_credential_that_was_never_connected_is_missing_its_token() {
        let error = auth_from_credential(
            "oauth1",
            &fields(json!({ "consumer_key": "ck", "consumer_secret": "cs" })),
        )
        .unwrap_err();

        assert_eq!(error, AuthFromCredentialError::missing("oauth_token"));
    }

    #[test]
    fn a_header_replaces_one_of_the_same_name_in_any_case() {
        let mut prepared = request("http://example.invalid/");
        prepared
            .headers
            .push(("Authorization".to_owned(), "old".to_owned()));
        prepared
            .headers
            .push(("X-Other".to_owned(), "kept".to_owned()));

        prepared.set_header("authorization", "new".to_owned());

        assert_eq!(prepared.headers.len(), 2);
        assert_eq!(header(&prepared, "AUTHORIZATION"), Some("new"));
        assert_eq!(header(&prepared, "x-other"), Some("kept"));
    }

    #[test]
    fn a_query_key_is_added_and_replaces_one_of_the_same_name() {
        let mut prepared = request("http://example.invalid/path?a=1&api_key=old&b=x%20y");

        prepared.set_query("api_key", "new value");

        let pairs: Vec<(String, String)> = prepared
            .url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("a".to_owned(), "1".to_owned()),
                ("b".to_owned(), "x y".to_owned()),
                ("api_key".to_owned(), "new value".to_owned()),
            ]
        );
        assert_eq!(prepared.url.path(), "/path");
    }

    #[test]
    fn a_query_credential_puts_its_value_in_the_url_and_nowhere_else() {
        let mut prepared = request("http://example.invalid/path");

        HttpAuth::Query {
            name: "api_key".to_owned(),
            value: "s3cret".to_owned(),
        }
        .prepare(&mut prepared)
        .unwrap();

        assert_eq!(prepared.url.query(), Some("api_key=s3cret"));
        assert!(prepared.headers.is_empty());
        assert_eq!(prepared.body, None);
    }

    #[test]
    fn a_custom_credential_adds_headers_query_and_body_fields() {
        let custom = HttpAuth::Custom(CustomAuth {
            headers: vec![("X-Token".to_owned(), "abc".to_owned())],
            query: vec![("sig".to_owned(), "z".to_owned())],
            body: fields(json!({ "client": "mestier" })),
        });

        let mut without_body = request("http://example.invalid/");
        custom.prepare(&mut without_body).unwrap();
        assert_eq!(header(&without_body, "x-token"), Some("abc"));
        assert_eq!(without_body.url.query(), Some("sig=z"));
        assert_eq!(without_body.body, Some(json!({ "client": "mestier" })));

        let mut with_body = request("http://example.invalid/");
        with_body.body = Some(json!({ "client": "other", "keep": 1 }));
        custom.prepare(&mut with_body).unwrap();
        assert_eq!(
            with_body.body,
            Some(json!({ "client": "mestier", "keep": 1 }))
        );

        let mut array_body = request("http://example.invalid/");
        array_body.body = Some(json!([1, 2]));
        let error = custom.prepare(&mut array_body).unwrap_err();
        assert!(error.contains("not an object"), "{error}");
    }

    #[test]
    fn a_custom_credential_without_body_fields_leaves_any_body_alone() {
        let custom = HttpAuth::Custom(CustomAuth {
            headers: Vec::new(),
            query: Vec::new(),
            body: Map::new(),
        });
        let mut prepared = request("http://example.invalid/");
        prepared.body = Some(json!([1, 2]));

        custom.prepare(&mut prepared).unwrap();

        assert_eq!(prepared.body, Some(json!([1, 2])));
    }

    #[test]
    fn a_jwt_credential_sends_a_signed_token_as_a_bearer() {
        let mut prepared = request("http://example.invalid/");

        HttpAuth::Jwt {
            algorithm: "HS256".to_owned(),
            key: "shared".to_owned(),
        }
        .prepare(&mut prepared)
        .unwrap();

        let value = header(&prepared, "authorization").unwrap();
        let token = value.strip_prefix("Bearer ").expect("a bearer");
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.required_spec_claims.clear();
        assert!(
            jsonwebtoken::decode::<Value>(
                token,
                &jsonwebtoken::DecodingKey::from_secret(b"shared"),
                &validation
            )
            .is_ok()
        );
    }

    #[test]
    fn an_oauth1_credential_signs_the_request_in_the_authorization_header() {
        let mut prepared = request("https://api.example.com/things?page=2");

        HttpAuth::OAuth1(OAuth1Auth {
            consumer_key: "ck".to_owned(),
            consumer_secret: "cs".to_owned(),
            token: "tok".to_owned(),
            token_secret: "ts".to_owned(),
            signature_method: "HMAC-SHA256".to_owned(),
        })
        .prepare(&mut prepared)
        .unwrap();

        let value = header(&prepared, "authorization").unwrap();
        assert!(value.starts_with("OAuth "), "{value}");
        for expected in [
            r#"oauth_consumer_key="ck""#,
            r#"oauth_token="tok""#,
            r#"oauth_signature_method="HMAC-SHA256""#,
            "oauth_signature=",
            "oauth_nonce=",
            "oauth_timestamp=",
        ] {
            assert!(value.contains(expected), "{expected}: {value}");
        }
        assert!(
            !value.contains("cs") || !value.contains("consumer_secret"),
            "{value}"
        );
        assert_eq!(prepared.url.query(), Some("page=2"));
    }

    fn sha256_hex(text: &str) -> String {
        aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, text.as_bytes())
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn parameter(header: &str, name: &str) -> String {
        let marker = format!("{name}=");
        let start = header.find(&marker).expect(name) + marker.len();
        let rest = &header[start..];
        match rest.strip_prefix('"') {
            Some(quoted) => quoted.split('"').next().unwrap().to_owned(),
            None => rest.split(',').next().unwrap().trim().to_owned(),
        }
    }

    #[tokio::test]
    async fn a_digest_credential_answers_the_challenge_once_with_a_valid_response() {
        let server = TestServer::start(|request| {
            if request.headers.contains_key("authorization") {
                StubResponse::text(200, "welcome")
            } else {
                StubResponse::text(401, "no").with_header(
                    "www-authenticate",
                    r#"Digest realm="api", nonce="abc123", qop="auth", algorithm=SHA-256, opaque="xyz""#,
                )
            }
        })
        .await;
        let auth = HttpAuth::Digest {
            username: "bot".to_owned(),
            password: "pw".to_owned(),
        };
        let prepared = request(&format!("{}/items?limit=2", server.url));

        let response = send(
            &Client::new(),
            &prepared,
            Duration::from_secs(5),
            Some(&auth),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), 200);
        let requests = server.requests();
        assert_eq!(requests.len(), 2, "one challenge, one retry");
        assert!(!requests[0].headers.contains_key("authorization"));
        let header = &requests[1].headers["authorization"];
        assert!(header.starts_with("Digest "), "{header}");
        assert_eq!(parameter(header, "username"), "bot");
        assert_eq!(parameter(header, "uri"), "/items?limit=2");
        assert_eq!(parameter(header, "opaque"), "xyz");
        let first = sha256_hex("bot:api:pw");
        let second = sha256_hex("GET:/items?limit=2");
        let expected = sha256_hex(&format!(
            "{first}:abc123:{}:{}:auth:{second}",
            parameter(header, "nc"),
            parameter(header, "cnonce"),
        ));
        assert_eq!(parameter(header, "response"), expected);
    }

    #[tokio::test]
    async fn a_digest_challenge_is_answered_once_even_when_the_answer_is_refused() {
        let server = TestServer::respond_always(StubResponse::text(401, "no").with_header(
            "www-authenticate",
            r#"Digest realm="api", nonce="abc123", qop="auth""#,
        ))
        .await;
        let auth = HttpAuth::Digest {
            username: "bot".to_owned(),
            password: "wrong".to_owned(),
        };

        let response = send(
            &Client::new(),
            &request(&server.url),
            Duration::from_secs(5),
            Some(&auth),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), 401);
        assert_eq!(server.requests().len(), 2);
    }

    #[tokio::test]
    async fn a_401_without_a_digest_challenge_or_for_another_credential_is_not_retried() {
        let basic_only = TestServer::respond_always(
            StubResponse::text(401, "no").with_header("www-authenticate", r#"Basic realm="api""#),
        )
        .await;
        let digest = HttpAuth::Digest {
            username: "bot".to_owned(),
            password: "pw".to_owned(),
        };
        let response = send(
            &Client::new(),
            &request(&basic_only.url),
            Duration::from_secs(5),
            Some(&digest),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), 401);
        assert_eq!(basic_only.requests().len(), 1);

        let digest_server = TestServer::respond_always(StubResponse::text(401, "no").with_header(
            "www-authenticate",
            r#"Digest realm="api", nonce="n", qop="auth""#,
        ))
        .await;
        let bearer = HttpAuth::Bearer {
            token: "t".to_owned(),
        };
        let mut prepared = request(&digest_server.url);
        bearer.prepare(&mut prepared).unwrap();
        let response = send(
            &Client::new(),
            &prepared,
            Duration::from_secs(5),
            Some(&bearer),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), 401);
        assert_eq!(digest_server.requests().len(), 1);
    }

    #[test]
    fn a_client_certificate_builds_a_client_and_other_auths_reuse_the_base() {
        let certificate = HttpAuth::ClientCertificate(ClientCertificate {
            certificate: TEST_CERTIFICATE.to_owned(),
            private_key: TEST_PRIVATE_KEY.to_owned(),
            passphrase: None,
            ca_certificate: Some(TEST_CERTIFICATE.to_owned()),
        });

        assert!(
            client_for_auth(
                &Client::new(),
                Some(PrivateNetworkAccess::Denied),
                Some(&certificate)
            )
            .is_ok()
        );
        assert!(client_for_auth(&Client::new(), None, Some(&certificate)).is_ok());
        assert!(client_for_auth(&Client::new(), None, None).is_ok());
        assert!(
            client_for_auth(
                &Client::new(),
                None,
                Some(&HttpAuth::Bearer {
                    token: "t".to_owned()
                })
            )
            .is_ok()
        );
    }

    #[test]
    fn an_unusable_client_certificate_is_refused_with_a_clear_message() {
        let with = |certificate: &str, key: &str, ca: Option<&str>| {
            HttpAuth::ClientCertificate(ClientCertificate {
                certificate: certificate.to_owned(),
                private_key: key.to_owned(),
                passphrase: None,
                ca_certificate: ca.map(str::to_owned),
            })
        };

        let garbage =
            client_for_auth(&Client::new(), None, Some(&with("nope", "nope", None))).unwrap_err();
        assert!(garbage.contains("client certificate"), "{garbage}");

        let encrypted = client_for_auth(
            &Client::new(),
            None,
            Some(&with(
                TEST_CERTIFICATE,
                "-----BEGIN ENCRYPTED PRIVATE KEY-----\nAAAA\n-----END ENCRYPTED PRIVATE KEY-----",
                None,
            )),
        )
        .unwrap_err();
        assert!(encrypted.contains("encrypted"), "{encrypted}");

        let bad_authority = client_for_auth(
            &Client::new(),
            None,
            Some(&with(
                TEST_CERTIFICATE,
                TEST_PRIVATE_KEY,
                Some("not a certificate"),
            )),
        )
        .unwrap_err();
        assert!(
            bad_authority.contains("certificate authority"),
            "{bad_authority}"
        );
    }
}
