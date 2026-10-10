use std::{future::Future, time::Duration};

use mcp_tools::Tool;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::verifier::Principal;

const PDP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, thiserror::Error)]
#[error("policy decision point unavailable")]
pub struct PdpError;

pub trait PolicyDecisionPoint {
    fn decide(
        &self,
        principal: &Principal,
        tool: Tool,
        organization: Option<&str>,
    ) -> impl Future<Output = Result<bool, PdpError>> + Send;
}

#[derive(Debug, Serialize)]
struct EvaluationRequest<'a> {
    subject: Subject<'a>,
    action: Action<'a>,
    resource: Resource<'a>,
    context: Map<String, Value>,
}

#[derive(Debug, Serialize)]
struct Subject<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    id: &'a str,
    properties: Value,
}

#[derive(Debug, Serialize)]
struct Action<'a> {
    name: &'a str,
}

#[derive(Debug, Serialize)]
struct Resource<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    id: &'a str,
    properties: Value,
}

#[derive(Debug, Deserialize)]
struct EvaluationResponse {
    decision: bool,
}

pub struct AuthzenPdp {
    http: Client,
    evaluation_url: String,
}

impl AuthzenPdp {
    pub fn new(http: Client, base_url: &str) -> Self {
        Self {
            http,
            evaluation_url: format!("{}/access/v1/evaluation", base_url.trim_end_matches('/')),
        }
    }
}

impl PolicyDecisionPoint for AuthzenPdp {
    async fn decide(
        &self,
        principal: &Principal,
        tool: Tool,
        organization: Option<&str>,
    ) -> Result<bool, PdpError> {
        let request = EvaluationRequest {
            subject: Subject {
                kind: "user",
                id: &principal.sub,
                properties: json!({
                    "roles": ["user"],
                    "client_id": principal.client_id,
                    "scopes": principal.scopes,
                }),
            },
            action: Action { name: tool.name() },
            resource: Resource {
                kind: "mcp_tool",
                id: tool.name(),
                properties: json!({
                    "access": tool.access().as_str(),
                    "domain": tool.domain(),
                    "organization": organization,
                }),
            },
            context: Map::new(),
        };
        let response = self
            .http
            .post(&self.evaluation_url)
            .timeout(PDP_TIMEOUT)
            .json(&request)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|_| PdpError)?;
        let body: EvaluationResponse = response.json().await.map_err(|_| PdpError)?;
        Ok(body.decision)
    }
}
