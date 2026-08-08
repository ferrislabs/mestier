use chrono::{DateTime, Utc};
use mestier_core::{CreatedWebhookEndpoint, DeliveryRecord, WebhookEndpoint};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Never carries the secret. The type is the guarantee: there is no field to
/// forget to strip.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct WebhookEndpointResponse {
    pub id: Uuid,
    pub url: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub event_names: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
}

impl WebhookEndpointResponse {
    pub fn new(endpoint: WebhookEndpoint, event_names: Vec<String>) -> Self {
        Self {
            id: endpoint.id,
            url: endpoint.url,
            description: endpoint.description,
            enabled: endpoint.enabled,
            event_names,
            created_at: endpoint.created_at,
            updated_at: endpoint.updated_at,
            disabled_at: endpoint.disabled_at,
        }
    }
}

/// The only response that ever carries a secret, returned once at creation.
/// Losing it means regenerating, not reading.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct CreatedWebhookEndpointResponse {
    #[serde(flatten)]
    pub endpoint: WebhookEndpointResponse,
    /// Shown once. It is not stored in a form anyone can read back.
    pub secret: String,
}

impl CreatedWebhookEndpointResponse {
    pub fn new(created: CreatedWebhookEndpoint, event_names: Vec<String>) -> Self {
        Self {
            endpoint: WebhookEndpointResponse::new(created.endpoint, event_names),
            secret: created.secret,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct SecretResponse {
    pub secret: String,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AutomationSettingsResponse {
    pub event_retention_days: u64,
    pub succeeded_delivery_retention_days: u64,
    pub retry_schedule_seconds: Vec<u64>,
    pub disable_target_after: Option<u32>,
}

impl From<mestier_core::AutomationSettings> for AutomationSettingsResponse {
    fn from(value: mestier_core::AutomationSettings) -> Self {
        Self {
            event_retention_days: value.event_retention.as_secs() / 86_400,
            succeeded_delivery_retention_days: value.succeeded_delivery_retention.as_secs()
                / 86_400,
            retry_schedule_seconds: value
                .retry_schedule
                .iter()
                .map(|interval| interval.as_secs())
                .collect(),
            disable_target_after: value.disable_target_after,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DeliveryResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub event_name: String,
    pub status: String,
    pub attempts: i32,
    pub next_attempt_at: DateTime<Utc>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<DeliveryRecord> for DeliveryResponse {
    fn from(value: DeliveryRecord) -> Self {
        Self {
            id: value.id,
            event_id: value.event_id,
            event_name: value.event_name,
            status: value.status,
            attempts: value.attempts,
            next_attempt_at: value.next_attempt_at,
            last_error: value.last_error,
            created_at: value.created_at,
            completed_at: value.completed_at,
        }
    }
}

/// One event an endpoint may subscribe to, straight from the catalogue.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct EventDescriptorResponse {
    pub name: String,
    pub version: u16,
    pub label: String,
    pub subject_kind: String,
    pub payload_example: serde_json::Value,
}
