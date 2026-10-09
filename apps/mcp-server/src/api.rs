use std::fmt;

use reqwest::StatusCode;
use serde_json::Value;

pub const MAX_PDF_BYTES: usize = 10 * 1024 * 1024;

/// A failure shown to the model as a tool error. The text is what the API
/// said, French included, so the user can act on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault(pub String);

impl Fault {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Fault {}

/// Thin HTTP client for the Mestier API. The bearer token is a parameter of
/// every call and is never kept.
#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base: String,
}

impl ApiClient {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base: base.into().trim_end_matches('/').to_owned(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1/{}", self.base, path.trim_start_matches('/'))
    }

    pub async fn get(
        &self,
        token: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Value, Fault> {
        let request = self.http.get(self.url(path)).query(query);
        self.json(request, token).await
    }

    pub async fn post(&self, token: &str, path: &str, body: &Value) -> Result<Value, Fault> {
        let request = self.http.post(self.url(path)).json(body);
        self.json(request, token).await
    }

    pub async fn put(&self, token: &str, path: &str, body: &Value) -> Result<Value, Fault> {
        let request = self.http.put(self.url(path)).json(body);
        self.json(request, token).await
    }

    /// Returns `None` when the body exceeds `limit`, without reading the rest.
    pub async fn get_bytes(
        &self,
        token: &str,
        path: &str,
        limit: usize,
    ) -> Result<Option<Vec<u8>>, Fault> {
        let response = self
            .http
            .get(self.url(path))
            .bearer_auth(token)
            .send()
            .await
            .map_err(unreachable_api)?;
        let response = check(response).await?;
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Ok(None);
        }
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(unreachable_api)? {
            if bytes.len() + chunk.len() > limit {
                return Ok(None);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(Some(bytes))
    }

    async fn json(&self, request: reqwest::RequestBuilder, token: &str) -> Result<Value, Fault> {
        let response = request
            .bearer_auth(token)
            .send()
            .await
            .map_err(unreachable_api)?;
        let response = check(response).await?;
        response
            .json()
            .await
            .map_err(|_| Fault::new("réponse illisible de l'API Mestier"))
    }
}

fn unreachable_api(error: reqwest::Error) -> Fault {
    tracing::warn!(%error, "Mestier API call failed");
    Fault::new("API Mestier injoignable")
}

async fn check(response: reqwest::Response) -> Result<reqwest::Response, Fault> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    if status == StatusCode::UNAUTHORIZED {
        return Err(Fault::new("session expirée"));
    }
    let body: Value = response.json().await.unwrap_or(Value::Null);
    let message = body
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("erreur {} de l'API Mestier", status.as_u16()));
    Err(Fault(message))
}
