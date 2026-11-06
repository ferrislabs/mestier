use std::time::Duration;

use aws_lc_rs::{
    digest,
    rand::{SecureRandom, SystemRandom},
};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use reqwest::{
    Client, Url,
    header::{ACCEPT, AUTHORIZATION},
};
use serde_json::{Map, Value};

use super::connectors::{
    http_client::{read_response, refuse_literal_private_address},
    oauth1::{self, percent_encode},
};
use super::webhook::address_policy::PrivateNetworkAccess;

pub const PROVIDER_TIMEOUT: Duration = Duration::from_secs(10);

pub fn random_token(bytes: usize) -> String {
    let mut buffer = vec![0u8; bytes];
    SystemRandom::new()
        .fill(&mut buffer)
        .expect("the system random source answers");
    URL_SAFE_NO_PAD.encode(buffer)
}

pub fn state_hash(state: &str) -> Vec<u8> {
    digest::digest(&digest::SHA256, state.as_bytes())
        .as_ref()
        .to_vec()
}

pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(digest::digest(&digest::SHA256, verifier.as_bytes()).as_ref())
}

fn text(fields: &Map<String, Value>, name: &str) -> Option<String> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn required(fields: &Map<String, Value>, name: &str) -> Result<String, String> {
    text(fields, name).ok_or_else(|| format!("the credential is missing field `{name}`"))
}

#[derive(Debug, Clone, PartialEq)]
pub struct OAuth2Settings {
    pub authorization_url: Option<String>,
    pub token_url: String,
    pub userinfo_url: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Option<String>,
    pub pkce: bool,
    pub credentials_in_header: bool,
}

impl OAuth2Settings {
    pub fn from_fields(fields: &Map<String, Value>) -> Result<Self, String> {
        Ok(Self {
            authorization_url: text(fields, "authorization_url"),
            token_url: required(fields, "token_url")?,
            userinfo_url: text(fields, "userinfo_url"),
            client_id: required(fields, "client_id")?,
            client_secret: required(fields, "client_secret")?,
            scopes: text(fields, "scopes"),
            pkce: text(fields, "pkce").as_deref() != Some("no"),
            credentials_in_header: text(fields, "client_authentication").as_deref() != Some("body"),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum Grant<'a> {
    AuthorizationCode {
        code: &'a str,
        redirect_uri: &'a str,
        verifier: Option<&'a str>,
    },
    Refresh {
        refresh_token: &'a str,
    },
    ClientCredentials,
}

pub fn authorization_url(
    settings: &OAuth2Settings,
    redirect_uri: &str,
    state: &str,
    challenge: Option<&str>,
) -> Result<Url, String> {
    let base = settings
        .authorization_url
        .as_deref()
        .ok_or_else(|| "the credential is missing field `authorization_url`".to_owned())?;
    let mut url =
        Url::parse(base).map_err(|error| format!("the authorization url is invalid: {error}"))?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("response_type", "code");
        pairs.append_pair("client_id", &settings.client_id);
        pairs.append_pair("redirect_uri", redirect_uri);
        if let Some(scopes) = &settings.scopes {
            pairs.append_pair("scope", scopes);
        }
        pairs.append_pair("state", state);
        if let Some(challenge) = challenge {
            pairs.append_pair("code_challenge", challenge);
            pairs.append_pair("code_challenge_method", "S256");
        }
    }
    Ok(url)
}

fn guard(url: &str, access: Option<PrivateNetworkAccess>) -> Result<Url, String> {
    let parsed = Url::parse(url).map_err(|error| format!("`{url}` is not a valid URL: {error}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("the provider url must use http or https".to_owned());
    }
    if let Some(access) = access {
        refuse_literal_private_address(&parsed, access)?;
    }
    Ok(parsed)
}

fn provider_error_code(body: &Value) -> Option<String> {
    body.get("error")
        .and_then(Value::as_str)
        .filter(|code| {
            code.len() <= 64
                && code
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))
        })
        .map(str::to_owned)
}

pub fn parse_token_response(status: u16, body: &Value) -> Result<TokenSet, String> {
    if !(200..300).contains(&status) {
        return Err(match provider_error_code(body) {
            Some(code) => format!("the token endpoint answered {status} ({code})"),
            None => format!("the token endpoint answered {status}"),
        });
    }
    let access_token = body
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| "the token endpoint answered without an access token".to_owned())?
        .to_owned();
    let refresh_token = body
        .get("refresh_token")
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty())
        .map(str::to_owned);
    let expires_in = match body.get("expires_in") {
        Some(Value::Number(number)) => number.as_u64(),
        Some(Value::String(text)) => text.trim().parse().ok(),
        _ => None,
    };
    Ok(TokenSet {
        access_token,
        refresh_token,
        expires_in,
    })
}

pub async fn request_tokens(
    client: &Client,
    access: Option<PrivateNetworkAccess>,
    settings: &OAuth2Settings,
    grant: Grant<'_>,
) -> Result<TokenSet, String> {
    let url = guard(&settings.token_url, access)?;
    let mut form: Vec<(&str, String)> = Vec::new();
    match grant {
        Grant::AuthorizationCode {
            code,
            redirect_uri,
            verifier,
        } => {
            form.push(("grant_type", "authorization_code".to_owned()));
            form.push(("code", code.to_owned()));
            form.push(("redirect_uri", redirect_uri.to_owned()));
            if let Some(verifier) = verifier {
                form.push(("code_verifier", verifier.to_owned()));
            }
        }
        Grant::Refresh { refresh_token } => {
            form.push(("grant_type", "refresh_token".to_owned()));
            form.push(("refresh_token", refresh_token.to_owned()));
        }
        Grant::ClientCredentials => {
            form.push(("grant_type", "client_credentials".to_owned()));
            if let Some(scopes) = &settings.scopes {
                form.push(("scope", scopes.clone()));
            }
        }
    }

    let mut request = client
        .post(url)
        .timeout(PROVIDER_TIMEOUT)
        .header(ACCEPT, "application/json");
    let in_header = matches!(grant, Grant::ClientCredentials) && settings.credentials_in_header;
    if in_header {
        let credentials = format!(
            "{}:{}",
            percent_encode(&settings.client_id),
            percent_encode(&settings.client_secret)
        );
        request = request.header(
            AUTHORIZATION,
            format!("Basic {}", STANDARD.encode(credentials)),
        );
    } else {
        form.push(("client_id", settings.client_id.clone()));
        form.push(("client_secret", settings.client_secret.clone()));
    }

    let response = request.form(&form).send().await.map_err(|error| {
        format!(
            "the token endpoint could not be reached: {}",
            describe(&error)
        )
    })?;
    let captured = read_response(response).await?;
    parse_token_response(captured.status, &captured.body)
}

fn describe(error: &reqwest::Error) -> String {
    super::connectors::http_client::describe_reqwest_error(error)
}

pub fn account_from_userinfo(body: &Value) -> Option<String> {
    let object = body.as_object()?;
    ["email", "preferred_username", "name", "login", "sub"]
        .iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|account| !account.is_empty())
        .map(|account| account.chars().take(200).collect())
}

pub async fn fetch_account(
    client: &Client,
    access: Option<PrivateNetworkAccess>,
    userinfo_url: &str,
    access_token: &str,
) -> Option<String> {
    let url = guard(userinfo_url, access).ok()?;
    let response = client
        .get(url)
        .timeout(PROVIDER_TIMEOUT)
        .header(ACCEPT, "application/json")
        .bearer_auth(access_token)
        .send()
        .await
        .ok()?;
    let captured = read_response(response).await.ok()?;
    if !(200..300).contains(&captured.status) {
        return None;
    }
    account_from_userinfo(&captured.body)
}

#[derive(Debug, Clone, PartialEq)]
pub struct OAuth1Settings {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub request_token_url: String,
    pub authorization_url: String,
    pub access_token_url: String,
    pub signature_method: String,
}

impl OAuth1Settings {
    pub fn from_fields(fields: &Map<String, Value>) -> Result<Self, String> {
        Ok(Self {
            consumer_key: required(fields, "consumer_key")?,
            consumer_secret: required(fields, "consumer_secret")?,
            request_token_url: required(fields, "request_token_url")?,
            authorization_url: required(fields, "authorization_url")?,
            access_token_url: required(fields, "access_token_url")?,
            signature_method: text(fields, "signature_method")
                .unwrap_or_else(|| "HMAC-SHA1".to_owned()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth1Token {
    pub token: String,
    pub secret: String,
}

fn parse_oauth1_response(status: u16, body: &Value) -> Result<OAuth1Token, String> {
    if !(200..300).contains(&status) {
        return Err(format!("the provider answered {status}"));
    }
    let encoded = body
        .as_str()
        .ok_or_else(|| "the provider answered in an unexpected format".to_owned())?;
    let parsed = Url::parse(&format!("http://provider.invalid/?{encoded}"))
        .map_err(|_| "the provider answered in an unexpected format".to_owned())?;
    let find = |name: &str| {
        parsed
            .query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
            .filter(|value| !value.is_empty())
    };
    match (find("oauth_token"), find("oauth_token_secret")) {
        (Some(token), Some(secret)) => Ok(OAuth1Token { token, secret }),
        _ => Err("the provider answered without a token".to_owned()),
    }
}

async fn signed_post(
    client: &Client,
    access: Option<PrivateNetworkAccess>,
    settings: &OAuth1Settings,
    url: &str,
    token: Option<&OAuth1Token>,
    extra: &[(String, String)],
) -> Result<OAuth1Token, String> {
    let url = guard(url, access)?;
    let signer = oauth1::Signer {
        consumer_key: &settings.consumer_key,
        consumer_secret: &settings.consumer_secret,
        token: token.map(|token| token.token.as_str()),
        token_secret: token.map(|token| token.secret.as_str()),
        signature_method: &settings.signature_method,
    };
    let header = oauth1::authorization_header(
        &signer,
        "POST",
        &url,
        extra,
        &random_token(16),
        chrono::Utc::now().timestamp().max(0) as u64,
    )?;
    let response = client
        .post(url)
        .timeout(PROVIDER_TIMEOUT)
        .header(AUTHORIZATION, header)
        .send()
        .await
        .map_err(|error| format!("the provider could not be reached: {}", describe(&error)))?;
    let captured = read_response(response).await?;
    parse_oauth1_response(captured.status, &captured.body)
}

pub async fn request_oauth1_token(
    client: &Client,
    access: Option<PrivateNetworkAccess>,
    settings: &OAuth1Settings,
    callback: &str,
) -> Result<OAuth1Token, String> {
    signed_post(
        client,
        access,
        settings,
        &settings.request_token_url,
        None,
        &[("oauth_callback".to_owned(), callback.to_owned())],
    )
    .await
}

pub async fn exchange_oauth1_token(
    client: &Client,
    access: Option<PrivateNetworkAccess>,
    settings: &OAuth1Settings,
    request_token: &OAuth1Token,
    verifier: &str,
) -> Result<OAuth1Token, String> {
    signed_post(
        client,
        access,
        settings,
        &settings.access_token_url,
        Some(request_token),
        &[("oauth_verifier".to_owned(), verifier.to_owned())],
    )
    .await
}

pub fn oauth1_authorization_url(
    settings: &OAuth1Settings,
    request_token: &str,
) -> Result<Url, String> {
    let mut url = Url::parse(&settings.authorization_url)
        .map_err(|error| format!("the authorization url is invalid: {error}"))?;
    url.query_pairs_mut()
        .append_pair("oauth_token", request_token);
    Ok(url)
}
