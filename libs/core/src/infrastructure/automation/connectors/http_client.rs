//! Shared HTTP plumbing for the network connectors (`http.request` and the
//! `odoo.*` family): a guarded client builder, the three auth schemes
//! `http.request` accepts, and how a response becomes what a connector
//! produces. Kept separate from `http_request.rs` because the `odoo.*`
//! connectors are, per #202, "a typed envelope on top of an HTTP call" —
//! they reuse this, not `http.request` itself.

use std::time::Duration;

use common::OrganizationId;
use reqwest::{Client, Response, Url, redirect::Policy};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::application::MestierUseCase;
use crate::domain::automation::credential::CredentialStatus;
use crate::domain::automation::run::ConnectorOutcome;
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
        .no_proxy()
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
    let literal = host
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .unwrap_or(host);
    let Ok(address) = literal.parse::<std::net::IpAddr>() else {
        return Ok(());
    };

    match judge(address, access) {
        AddressVerdict::Allowed => Ok(()),
        AddressVerdict::Refused(reason) => Err(format!("`{host}` is {reason}")),
    }
}

pub use super::http_auth::{
    HttpAuth, PreparedRequest, auth_from_credential, client_for_auth, send,
};
use crate::infrastructure::automation::webhook::inbound::{SignatureAlgorithm, SignatureSpec};

#[derive(Debug, Clone, PartialEq)]
pub enum InboundAuth {
    Headers(HttpAuth),
    Signature(SignatureSpec),
}

pub async fn open_http_auth(
    usecase: &MestierUseCase,
    org_id: OrganizationId,
    credential_id: Uuid,
) -> Result<HttpAuth, String> {
    let (credential, plaintext) = usecase
        .open_credential_for_run(org_id, credential_id)
        .await?;

    if credential.status == CredentialStatus::NotConnected {
        return Err(format!("credential `{}` is not connected", credential.name));
    }

    if matches!(
        credential.kind.as_str(),
        "oauth2_client_credentials" | "oauth2_authorization_code"
    ) {
        let token = usecase.oauth_access_token(org_id, credential_id).await?;
        return Ok(HttpAuth::Bearer { token });
    }

    let fields = parse_credential_fields(&plaintext)
        .ok_or_else(|| "the auth credential's data is not usable".to_owned())?;

    auth_from_credential(&credential.kind, &fields).map_err(|error| error.to_string())
}

pub async fn open_webhook_auth(
    usecase: &MestierUseCase,
    org_id: OrganizationId,
    credential_id: Uuid,
) -> Result<InboundAuth, String> {
    let (credential, plaintext) = usecase
        .open_credential_for_run(org_id, credential_id)
        .await?;

    let fields = parse_credential_fields(&plaintext)
        .ok_or_else(|| "the auth credential's data is not usable".to_owned())?;

    if credential.kind == "hmac" {
        let text = |name: &str| fields.get(name).and_then(Value::as_str).map(str::to_owned);
        let algorithm = text("algorithm")
            .as_deref()
            .and_then(SignatureAlgorithm::parse)
            .ok_or_else(|| "the signature algorithm is not usable".to_owned())?;
        return Ok(InboundAuth::Signature(SignatureSpec {
            header_name: text("header_name")
                .ok_or_else(|| "the credential is missing field `header_name`".to_owned())?,
            algorithm,
            secret: text("secret")
                .ok_or_else(|| "the credential is missing field `secret`".to_owned())?,
        }));
    }

    auth_from_credential(&credential.kind, &fields)
        .map(InboundAuth::Headers)
        .map_err(|error| error.to_string())
}

fn parse_credential_fields(plaintext: &[u8]) -> Option<Map<String, Value>> {
    match serde_json::from_slice(plaintext).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

/// Only `2xx` is a success — a redirect (never followed, `Policy::none()`)
/// and every other status go back into the retry schedule, the body kept in
/// the message so the run inspector shows *why* rather than just the code.
pub fn outcome_for(captured: CapturedResponse) -> ConnectorOutcome {
    if (200..300).contains(&captured.status) {
        ConnectorOutcome::Produced(json!({
            "status": captured.status,
            "headers": captured.headers,
            "body": captured.body,
        }))
    } else {
        ConnectorOutcome::Failed {
            error: format!(
                "the endpoint answered {} with body: {}",
                captured.status, captured.body
            ),
        }
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
/// on the way.
pub async fn read_response(mut response: Response) -> Result<CapturedResponse, String> {
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let headers = headers_to_value(response.headers());
    let declared_over_the_limit = response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BODY_BYTES as u64);

    let ceiling = MAX_RESPONSE_BODY_BYTES + 1;
    let mut collected: Vec<u8> = Vec::new();
    while collected.len() < ceiling {
        let chunk = response.chunk().await.map_err(|error| {
            format!(
                "reading the response body failed: {}",
                describe_reqwest_error(error)
            )
        })?;
        let Some(chunk) = chunk else {
            break;
        };
        let room = ceiling - collected.len();
        collected.extend_from_slice(&chunk[..chunk.len().min(room)]);
    }
    drop(response);

    let (capped, cut) = cap_body(&collected);
    let truncated = cut || declared_over_the_limit;
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
pub fn describe_reqwest_error(error: reqwest::Error) -> String {
    use std::error::Error;

    let error = error.without_url();
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

    use super::super::http_auth::AuthFromCredentialError;
    use super::*;

    #[tokio::test]
    async fn a_failed_call_is_described_without_the_url_it_carried() {
        let client = Client::builder().no_proxy().build().unwrap();

        let error = client
            .get("http://127.0.0.1:1/items?api_key=QUERY-SECRET-VALUE&sig=OTHER-SECRET")
            .send()
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("QUERY-SECRET-VALUE"),
            "the raw error does carry the url: that is the leak"
        );

        let described = describe_reqwest_error(error);

        assert!(!described.contains("QUERY-SECRET-VALUE"), "{described}");
        assert!(!described.contains("OTHER-SECRET"), "{described}");
        assert!(!described.contains("api_key"), "{described}");
        assert!(!described.is_empty());
    }

    fn fields(pairs: &[(&str, &str)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect()
    }

    #[test]
    #[ignore = "re-run by the proxy test in a child process"]
    fn proxy_probe_child() {
        let Ok(probe) = std::env::var("MESTIER_PROXY_PROBE_TARGET") else {
            return;
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let client = build_guarded_client(PrivateNetworkAccess::Allowed).unwrap();
            let _ = client
                .get(probe)
                .timeout(std::time::Duration::from_secs(2))
                .send()
                .await;
        });
    }

    #[tokio::test]
    async fn the_guarded_client_never_goes_through_a_proxy_from_the_environment() {
        use crate::infrastructure::automation::connectors::test_support::{
            StubResponse, TestServer,
        };

        let proxy = TestServer::respond_always(StubResponse::text(200, "proxied")).await;
        let test_name = format!(
            "{}::proxy_probe_child",
            module_path!().trim_start_matches("mestier_core::")
        );
        let executable = std::env::current_exe().unwrap();
        let proxy_url = proxy.url.clone();

        let status = tokio::task::spawn_blocking(move || {
            std::process::Command::new(executable)
                .args(["--exact", &test_name, "--ignored", "--nocapture"])
                .env("HTTP_PROXY", &proxy_url)
                .env("http_proxy", &proxy_url)
                .env("ALL_PROXY", &proxy_url)
                .env("MESTIER_PROXY_PROBE_TARGET", "http://203.0.113.7:9/probe")
                .output()
                .unwrap()
                .status
        })
        .await
        .unwrap();

        assert!(status.success());
        assert!(
            proxy.requests().is_empty(),
            "the call went through the proxy: {:?}",
            proxy.requests()
        );
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
    fn a_literal_ipv6_loopback_is_refused_whatever_the_policy() {
        let url = Url::parse("http://[::1]:9/probe").unwrap();

        for access in [PrivateNetworkAccess::Denied, PrivateNetworkAccess::Allowed] {
            let error = refuse_literal_private_address(&url, access)
                .expect_err("an IPv6 loopback must be refused");
            assert!(error.contains("loopback"), "{error}");
        }
    }

    #[test]
    fn a_mapped_ipv6_metadata_address_is_refused_whatever_the_policy() {
        let url = Url::parse("http://[::ffff:a9fe:a9fe]/latest/meta-data").unwrap();

        for access in [PrivateNetworkAccess::Denied, PrivateNetworkAccess::Allowed] {
            let error = refuse_literal_private_address(&url, access)
                .expect_err("the mapped metadata endpoint must be refused");
            assert!(error.contains("metadata"), "{error}");
        }
    }

    #[test]
    fn a_literal_ipv6_unique_local_address_is_refused_when_denied() {
        let url = Url::parse("http://[fd12:3456:789a::1]/").unwrap();

        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Denied).is_err());
        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Allowed).is_ok());
    }

    #[test]
    fn the_aws_ipv6_metadata_address_is_refused_whatever_the_policy() {
        let url = Url::parse("http://[fd00:ec2::254]/latest/meta-data").unwrap();

        for access in [PrivateNetworkAccess::Denied, PrivateNetworkAccess::Allowed] {
            let error = refuse_literal_private_address(&url, access)
                .expect_err("the AWS IPv6 metadata endpoint must be refused");
            assert!(error.contains("metadata"), "{error}");
        }
    }

    #[test]
    fn a_public_literal_ipv6_address_is_allowed() {
        let url = Url::parse("http://[2606:4700:4700::1111]/").unwrap();

        assert!(refuse_literal_private_address(&url, PrivateNetworkAccess::Denied).is_ok());
    }

    #[test]
    fn an_obfuscated_ipv4_literal_is_judged_on_the_address_it_means() {
        for spelled in [
            "http://2130706433/",
            "http://0x7f.1/",
            "http://017700000001/",
        ] {
            let url = Url::parse(spelled).unwrap();

            let error = refuse_literal_private_address(&url, PrivateNetworkAccess::Allowed)
                .expect_err(spelled);

            assert!(error.contains("loopback"), "{spelled}: {error}");
        }
    }

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

    fn prepared() -> PreparedRequest {
        PreparedRequest::new(
            reqwest::Method::GET,
            Url::parse("http://example.invalid/path").unwrap(),
            Vec::new(),
            None,
        )
    }

    fn header_of<'a>(request: &'a PreparedRequest, name: &str) -> Option<&'a str> {
        request
            .headers
            .iter()
            .find(|(existing, _)| existing.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    #[test]
    fn bearer_token_produces_an_authorization_bearer_header() {
        let auth = auth_from_credential("bearer_token", &fields(&[("token", "abc123")]))
            .expect("bearer_token has everything it needs");
        let mut request = prepared();

        auth.prepare(&mut request).unwrap();

        assert_eq!(header_of(&request, "authorization"), Some("Bearer abc123"));
    }

    #[test]
    fn http_basic_produces_a_base64_authorization_header() {
        let auth = auth_from_credential(
            "http_basic",
            &fields(&[("username", "bot"), ("password", "secret")]),
        )
        .expect("http_basic has everything it needs");
        let mut request = prepared();

        auth.prepare(&mut request).unwrap();

        let expected = format!("Basic {}", STANDARD.encode("bot:secret"));
        assert_eq!(
            header_of(&request, "authorization"),
            Some(expected.as_str())
        );
    }

    #[test]
    fn http_header_produces_the_named_custom_header() {
        let auth = auth_from_credential(
            "http_header",
            &fields(&[("header_name", "x-api-key"), ("header_value", "shh")]),
        )
        .expect("http_header has everything it needs");
        let mut request = prepared();

        auth.prepare(&mut request).unwrap();

        assert_eq!(header_of(&request, "x-api-key"), Some("shh"));
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

    async fn endless_server(declared_length: Option<u64>) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let mut request = [0u8; 4096];
            let _ = socket.read(&mut request).await;
            let head = match declared_length {
                Some(length) => format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ncontent-length: {length}\r\n\r\n"
                ),
                None => "HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ntransfer-encoding: chunked\r\n\r\n"
                    .to_owned(),
            };
            if socket.write_all(head.as_bytes()).await.is_err() {
                return;
            }
            let block = vec![b'a'; 64 * 1024];
            loop {
                let written = if declared_length.is_some() {
                    socket.write_all(&block).await
                } else {
                    let mut framed = format!("{:x}\r\n", block.len()).into_bytes();
                    framed.extend_from_slice(&block);
                    framed.extend_from_slice(b"\r\n");
                    socket.write_all(&framed).await
                };
                if written.is_err() {
                    return;
                }
            }
        });
        format!("http://{address}/")
    }

    async fn read_capped(url: String) -> Result<CapturedResponse, String> {
        let client = Client::builder().no_proxy().build().unwrap();
        let response = client.get(url).send().await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), read_response(response))
            .await
            .map_err(|_| "reading the body never stopped".to_owned())?
    }

    #[tokio::test]
    async fn a_body_declared_far_over_the_limit_is_cut_without_waiting_for_the_rest() {
        let url = endless_server(Some(50 * 1024 * 1024 * 1024)).await;

        let captured = read_capped(url).await.expect("the read stops at the limit");

        let body = captured.body.as_str().expect("a text body");
        assert!(body.ends_with(TRUNCATION_MARKER));
        assert_eq!(
            body.len(),
            MAX_RESPONSE_BODY_BYTES + TRUNCATION_MARKER.len()
        );
    }

    #[tokio::test]
    async fn a_body_of_unknown_length_that_never_ends_is_cut_at_the_limit() {
        let url = endless_server(None).await;

        let captured = read_capped(url).await.expect("the read stops at the limit");

        let body = captured.body.as_str().expect("a text body");
        assert!(body.ends_with(TRUNCATION_MARKER));
        assert_eq!(
            body.len(),
            MAX_RESPONSE_BODY_BYTES + TRUNCATION_MARKER.len()
        );
    }

    #[tokio::test]
    async fn a_body_exactly_at_the_limit_is_kept_whole() {
        use crate::infrastructure::automation::connectors::test_support::{
            StubResponse, TestServer,
        };

        let server = TestServer::respond_always(StubResponse::text(
            200,
            vec![b'b'; MAX_RESPONSE_BODY_BYTES],
        ))
        .await;

        let captured = read_capped(server.url.clone()).await.unwrap();

        let body = captured.body.as_str().unwrap();
        assert_eq!(body.len(), MAX_RESPONSE_BODY_BYTES);
        assert!(!body.ends_with(TRUNCATION_MARKER));
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
