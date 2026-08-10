//! Shared HTTP plumbing for the network connectors (`http.request` and the
//! `odoo.*` family): a guarded client builder, the three auth schemes
//! `http.request` accepts, and how a response becomes what a connector
//! produces. Kept separate from `http_request.rs` because the `odoo.*`
//! connectors are, per #202, "a typed envelope on top of an HTTP call" —
//! they reuse this, not `http.request` itself.

use std::time::Duration;

use reqwest::{Client, RequestBuilder, Response, Url, redirect::Policy};
use serde_json::{Map, Value};

use crate::infrastructure::automation::webhook::{
    address_policy::{AddressVerdict, PrivateNetworkAccess, judge},
    resolver::GuardedResolver,
};

/// A stranger's endpoint gets this long by default when a connector's config
/// leaves `timeout_seconds` unset.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// A response body past this size is truncated for the run's output — large
/// enough for any reasonable API payload, small enough that a misbehaving
/// endpoint cannot inflate a run's stored output without bound.
pub const MAX_RESPONSE_BODY_BYTES: usize = 1_048_576;

pub const TRUNCATION_MARKER: &str = "…[truncated]";

/// Builds the client every network connector sends through.
///
/// Redirects are never followed (`Policy::none()`): a legitimate connector
/// call has no use for one, and following it would re-open the hole the
/// guarded resolver closes — the redirect target is resolved by the same
/// client, and a 3xx to a fresh name is exactly the shape of an attempted
/// bypass. The DNS resolver is [`GuardedResolver`], so address filtering
/// happens on the address actually connected to, not on a separate lookup
/// that could answer differently.
pub fn build_guarded_client(access: PrivateNetworkAccess) -> Result<Client, String> {
    Client::builder()
        .redirect(Policy::none())
        .dns_resolver(GuardedResolver::new(access))
        .build()
        .map_err(|error| format!("building the HTTP client failed: {error}"))
}

/// The one case [`GuardedResolver`] cannot cover: a URL whose host is
/// already a literal IP address has nothing to resolve, so `reqwest` never
/// calls a custom `Resolve` for it at all — it connects to the parsed
/// address directly. A workflow's `url` field supplying a raw private
/// address would otherwise reach none of the address policy. Checked before
/// the request is ever sent; a hostname is left untouched here and judged
/// later, on the address it actually resolves to, by the guarded resolver.
pub fn refuse_literal_private_address(
    url: &Url,
    access: PrivateNetworkAccess,
) -> Result<(), String> {
    let Some(host) = url.host_str() else {
        return Ok(());
    };
    let Ok(address) = host.parse::<std::net::IpAddr>() else {
        return Ok(());
    };

    match judge(address, access) {
        AddressVerdict::Allowed => Ok(()),
        AddressVerdict::Refused(reason) => Err(format!("`{host}` is {reason}")),
    }
}

/// One of the three schemes `http.request` accepts
/// (`AuthRequirement::AnyOf(&["bearer_token", "http_basic", "http_header"])`),
/// carrying exactly the fields its auth scheme declares (see
/// `domain::automation::connector::scheme`).
#[derive(Debug, Clone, PartialEq)]
pub enum HttpAuth {
    Bearer { token: String },
    Basic { username: String, password: String },
    Header { name: String, value: String },
}

impl HttpAuth {
    pub fn apply(&self, builder: RequestBuilder) -> RequestBuilder {
        match self {
            HttpAuth::Bearer { token } => builder.bearer_auth(token),
            HttpAuth::Basic { username, password } => builder.basic_auth(username, Some(password)),
            HttpAuth::Header { name, value } => builder.header(name, value),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AuthFromCredentialError {
    #[error("unsupported auth credential kind `{kind}`")]
    UnsupportedKind { kind: String },
    #[error("credential is missing field `{field}`")]
    MissingField { field: String },
}

impl AuthFromCredentialError {
    #[cfg(test)]
    fn missing(field: &str) -> Self {
        Self::MissingField {
            field: field.to_owned(),
        }
    }
}

/// Turns an opened credential's fields (a Supplied credential's plaintext,
/// already validated against its scheme at creation time — see
/// `domain::automation::credential::validate_credential_data`) into the auth
/// this request applies.
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
        other => Err(AuthFromCredentialError::UnsupportedKind {
            kind: other.to_owned(),
        }),
    }
}

/// What a connector reads back once a response arrived: status, headers and
/// a body already turned into what the engine stores.
pub struct CapturedResponse {
    pub status: u16,
    pub headers: Value,
    pub body: Value,
}

/// Reads a response into what the connector produces, applying the size cap
/// on the way. The body is buffered whole before truncation — this crate's
/// `reqwest` has no `stream` feature enabled, so there is no cheaper way to
/// cap it without adding one.
pub async fn read_response(response: Response) -> Result<CapturedResponse, String> {
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let headers = headers_to_value(response.headers());

    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("reading the response body failed: {error}"))?;
    let (capped, truncated) = cap_body(&bytes);
    let body = interpret_body(content_type.as_deref(), &capped, truncated);

    Ok(CapturedResponse {
        status,
        headers,
        body,
    })
}

fn headers_to_value(headers: &reqwest::header::HeaderMap) -> Value {
    let mut map = Map::new();
    for (name, value) in headers {
        if let Ok(value) = value.to_str() {
            map.insert(name.as_str().to_owned(), Value::String(value.to_owned()));
        }
    }
    Value::Object(map)
}

/// `reqwest::Error`'s `Display` alone drops the source chain, which is where
/// the guarded resolver's refusal reason lives (`` `host` resolves to a
/// private address ``) — walking it is what lets a test, or an artisan
/// reading a run, see why a call never left the process. Shared by
/// `http.request` and the `odoo.*` connectors: both send through a guarded
/// client and both want the same depth of explanation for the same reason.
pub fn describe_reqwest_error(error: &reqwest::Error) -> String {
    use std::error::Error;

    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(err) = source {
        message.push_str(": ");
        message.push_str(&err.to_string());
        source = err.source();
    }
    message
}

/// Truncates `body` to [`MAX_RESPONSE_BODY_BYTES`], on a UTF-8 boundary so
/// the truncated bytes still decode to a displayable string.
fn cap_body(body: &[u8]) -> (Vec<u8>, bool) {
    if body.len() <= MAX_RESPONSE_BODY_BYTES {
        return (body.to_vec(), false);
    }

    let mut end = MAX_RESPONSE_BODY_BYTES;
    while end > 0 && std::str::from_utf8(&body[..end]).is_err() {
        end -= 1;
    }
    (body[..end].to_vec(), true)
}

/// JSON when the body actually parses as JSON, the raw string otherwise — a
/// truncated body is never handed to the JSON parser, since truncation
/// almost always makes it invalid and a parse failure there would be
/// misleading rather than informative.
fn interpret_body(content_type: Option<&str>, bytes: &[u8], truncated: bool) -> Value {
    let text = String::from_utf8_lossy(bytes).into_owned();
    let text = if truncated {
        format!("{text}{TRUNCATION_MARKER}")
    } else {
        text
    };

    if truncated {
        return Value::String(text);
    }

    let looks_like_json = content_type.is_some_and(|ct| ct.contains("json"));
    if (looks_like_json || text.trim_start().starts_with(['{', '[']))
        && let Ok(value) = serde_json::from_str::<Value>(&text)
    {
        return value;
    }

    Value::String(text)
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde_json::json;

    use super::*;

    fn fields(pairs: &[(&str, &str)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect()
    }

    #[test]
    fn a_literal_private_address_is_refused_when_denied() {
        let url = Url::parse("http://10.255.255.1:9/probe").unwrap();

        let error = refuse_literal_private_address(&url, PrivateNetworkAccess::Denied)
            .expect_err("a private address must be refused");

        assert!(error.contains("private"), "{error}");
    }

    #[test]
    fn a_literal_private_address_is_allowed_when_the_instance_permits_it() {
        let url = Url::parse("http://10.255.255.1:9/probe").unwrap();

        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Allowed).is_ok());
    }

    /// Loopback stays refused even when private network access is granted —
    /// the same discipline `judge` already enforces; this proves the
    /// preflight check calls it rather than reimplementing a weaker rule.
    #[test]
    fn a_literal_loopback_address_is_refused_even_when_allowed() {
        let url = Url::parse("http://127.0.0.1:9/probe").unwrap();

        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Allowed).is_err());
    }

    /// A hostname has nothing for this check to judge — it is left to the
    /// guarded resolver, which judges the address it actually resolves to.
    #[test]
    fn a_hostname_is_left_untouched_by_the_preflight_check() {
        let url = Url::parse("http://example.internal/probe").unwrap();

        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Denied).is_ok());
    }

    #[test]
    fn a_public_literal_address_is_allowed_either_way() {
        let url = Url::parse("http://93.184.216.34/probe").unwrap();

        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Denied).is_ok());
        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Allowed).is_ok());
    }

    #[test]
    fn bearer_token_produces_an_authorization_bearer_header() {
        let auth = auth_from_credential("bearer_token", &fields(&[("token", "abc123")]))
            .expect("bearer_token has everything it needs");
        let client = Client::new();

        let request = auth
            .apply(client.get("http://example.invalid"))
            .build()
            .unwrap();

        assert_eq!(
            request.headers().get("authorization").unwrap(),
            "Bearer abc123"
        );
    }

    #[test]
    fn http_basic_produces_a_base64_authorization_header() {
        let auth = auth_from_credential(
            "http_basic",
            &fields(&[("username", "bot"), ("password", "secret")]),
        )
        .expect("http_basic has everything it needs");
        let client = Client::new();

        let request = auth
            .apply(client.get("http://example.invalid"))
            .build()
            .unwrap();

        let expected = format!("Basic {}", STANDARD.encode("bot:secret"));
        assert_eq!(request.headers().get("authorization").unwrap(), &expected);
    }

    #[test]
    fn http_header_produces_the_named_custom_header() {
        let auth = auth_from_credential(
            "http_header",
            &fields(&[("header_name", "x-api-key"), ("header_value", "shh")]),
        )
        .expect("http_header has everything it needs");
        let client = Client::new();

        let request = auth
            .apply(client.get("http://example.invalid"))
            .build()
            .unwrap();

        assert_eq!(request.headers().get("x-api-key").unwrap(), "shh");
    }

    #[test]
    fn an_unsupported_kind_is_refused_and_named() {
        let error = auth_from_credential("odoo_api", &Map::new()).unwrap_err();

        assert_eq!(
            error,
            AuthFromCredentialError::UnsupportedKind {
                kind: "odoo_api".to_owned()
            }
        );
    }

    #[test]
    fn a_missing_field_is_refused_and_named() {
        let error = auth_from_credential("bearer_token", &Map::new()).unwrap_err();

        assert_eq!(error, AuthFromCredentialError::missing("token"));
    }

    #[test]
    fn a_body_within_the_cap_is_untouched() {
        let (capped, truncated) = cap_body(b"hello world");

        assert_eq!(capped, b"hello world");
        assert!(!truncated);
    }

    #[test]
    fn a_body_past_the_cap_is_truncated_and_flagged() {
        let body = vec![b'a'; MAX_RESPONSE_BODY_BYTES + 100];

        let (capped, truncated) = cap_body(&body);

        assert_eq!(capped.len(), MAX_RESPONSE_BODY_BYTES);
        assert!(truncated);
    }

    #[test]
    fn a_truncated_body_carries_its_marker_and_is_never_parsed_as_json() {
        let body = interpret_body(Some("application/json"), b"{\"a\":1", true);

        assert_eq!(body, Value::String(format!("{{\"a\":1{TRUNCATION_MARKER}")));
    }

    #[test]
    fn a_json_content_type_with_a_valid_body_is_parsed() {
        let body = interpret_body(Some("application/json"), br#"{"id":42}"#, false);

        assert_eq!(body, json!({ "id": 42 }));
    }

    #[test]
    fn a_json_looking_body_is_parsed_even_without_a_content_type() {
        let body = interpret_body(None, br#"{"id":42}"#, false);

        assert_eq!(body, json!({ "id": 42 }));
    }

    #[test]
    fn a_non_json_body_stays_a_string() {
        let body = interpret_body(Some("text/plain"), b"just text", false);

        assert_eq!(body, Value::String("just text".to_owned()));
    }

    #[test]
    fn a_body_that_only_looks_like_json_but_is_not_falls_back_to_a_string() {
        let body = interpret_body(Some("application/json"), b"{not json", false);

        assert_eq!(body, Value::String("{not json".to_owned()));
    }
}
