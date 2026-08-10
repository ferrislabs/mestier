//! Response (and, where the shape is identical both ways, request) DTOs
//! shared across the crate's submodules.
//!
//! The catalogue DTOs (`ConnectorDescriptorResponse`, `FieldResponse`, …)
//! exist so the connector and event catalogues can leave the crate as JSON
//! with a `utoipa::ToSchema` the OpenAPI document can describe — the domain
//! types themselves (`mestier_core::ConnectorDescriptor`, `Field`, …) derive
//! `Serialize` only, and their `&'static str`/`&'static [Field]` fields
//! cannot carry an owned, `Deserialize`-able shape anyway. Each conversion
//! mirrors the domain type's own default `Serialize` shape field for field,
//! so the JSON a caller sees is exactly what the domain-level tests
//! (`connector::descriptor`, `connector::catalogue`) already lock in.

use std::time::Duration;

use chrono::{DateTime, Utc};
use mestier_core::{
    AuthRequirement, AuthScheme, AutomationSettings, Credential, CredentialOrigin, Field,
    FieldKind, OrganizationId, Run, RunStatus, RunStep, SelectOption, StepStatus, VisibleWhen,
    Workflow, WorkflowVersion,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

// --- catalogues ----------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct SelectOptionResponse {
    pub value: String,
    pub label: String,
}

impl From<&SelectOption> for SelectOptionResponse {
    fn from(value: &SelectOption) -> Self {
        Self {
            value: value.value.to_owned(),
            label: value.label.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct VisibleWhenResponse {
    pub field: String,
    pub any_of: Vec<String>,
}

impl From<&VisibleWhen> for VisibleWhenResponse {
    fn from(value: &VisibleWhen) -> Self {
        Self {
            field: value.field.to_owned(),
            any_of: value.any_of.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
}

/// Mirrors `FieldKind`'s own default (externally tagged) `Serialize` shape:
/// `"Text"`, `"Number"`, `"Bool"`, `"Json"` as bare strings, `{ "Select": {
/// "options": [...] } }` for the one variant carrying data.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub enum FieldKindResponse {
    Text,
    Number,
    Bool,
    Select { options: Vec<SelectOptionResponse> },
    Json,
}

impl From<&FieldKind> for FieldKindResponse {
    fn from(value: &FieldKind) -> Self {
        match value {
            FieldKind::Text => Self::Text,
            FieldKind::Number => Self::Number,
            FieldKind::Bool => Self::Bool,
            FieldKind::Select { options } => Self::Select {
                options: options.iter().map(SelectOptionResponse::from).collect(),
            },
            FieldKind::Json => Self::Json,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct FieldResponse {
    pub name: String,
    pub label: String,
    pub required: bool,
    pub kind: FieldKindResponse,
    /// This field accepts a `{{ ... }}` expression.
    pub expression: bool,
    /// Masked in the UI: never send this field's value back once stored.
    pub secret: bool,
    pub visible_when: Option<VisibleWhenResponse>,
}

impl From<&Field> for FieldResponse {
    fn from(value: &Field) -> Self {
        Self {
            name: value.name.to_owned(),
            label: value.label.to_owned(),
            required: value.required,
            kind: FieldKindResponse::from(&value.kind),
            expression: value.expression,
            secret: value.secret,
            visible_when: value.visible_when.as_ref().map(VisibleWhenResponse::from),
        }
    }
}

/// Mirrors `AuthRequirement`'s own default `Serialize` shape: `"None"` bare,
/// `{ "Exactly": "..." }`, `{ "AnyOf": [...] }`.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub enum AuthRequirementResponse {
    None,
    Exactly(String),
    AnyOf(Vec<String>),
}

impl From<&AuthRequirement> for AuthRequirementResponse {
    fn from(value: &AuthRequirement) -> Self {
        match value {
            AuthRequirement::None => Self::None,
            AuthRequirement::Exactly(kind) => Self::Exactly((*kind).to_owned()),
            AuthRequirement::AnyOf(kinds) => {
                Self::AnyOf(kinds.iter().map(|k| (*k).to_owned()).collect())
            }
        }
    }
}

/// What one connector kind's descriptor looks like on the wire — enough for
/// the editor to render a form with no connector known by name in the
/// frontend's own code.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ConnectorDescriptorResponse {
    pub kind: String,
    pub version: u16,
    pub family: String,
    pub label: String,
    pub auth: AuthRequirementResponse,
    pub fields: Vec<FieldResponse>,
    pub output_example: Value,
}

impl From<&mestier_core::ConnectorDescriptor> for ConnectorDescriptorResponse {
    fn from(value: &mestier_core::ConnectorDescriptor) -> Self {
        Self {
            kind: value.kind.to_owned(),
            version: value.version,
            family: value.family.to_owned(),
            label: value.label.to_owned(),
            auth: AuthRequirementResponse::from(&value.auth),
            fields: value.fields.iter().map(FieldResponse::from).collect(),
            output_example: value.output_example.clone(),
        }
    }
}

/// An authentication scheme's field layout — what a credential's create
/// form renders, keyed by the `kind` a `ConnectorDescriptorResponse::auth`
/// names.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AuthSchemeResponse {
    pub kind: String,
    pub label: String,
    pub fields: Vec<FieldResponse>,
}

impl From<&AuthScheme> for AuthSchemeResponse {
    fn from(value: &AuthScheme) -> Self {
        Self {
            kind: value.kind.to_owned(),
            label: value.label.to_owned(),
            fields: value.fields.iter().map(FieldResponse::from).collect(),
        }
    }
}

/// The connector catalogue, alongside the auth schemes those connectors'
/// `auth` field names — folded into this one response rather than a third
/// route: #203 freezes exactly two catalogue endpoints
/// (`GET /connectors`, `GET /events`), and a credential's create form is
/// exactly as data-driven-critical as a connector's.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ConnectorsResponse {
    pub connectors: Vec<ConnectorDescriptorResponse>,
    pub auth_schemes: Vec<AuthSchemeResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EventDescriptorResponse {
    pub name: String,
    pub version: u16,
    pub label: String,
    pub subject_kind: String,
    pub payload_example: Value,
}

impl From<&events::EventDescriptor> for EventDescriptorResponse {
    fn from(value: &events::EventDescriptor) -> Self {
        Self {
            name: value.name.to_owned(),
            version: value.version,
            label: value.label.to_owned(),
            subject_kind: value.subject_kind.to_owned(),
            payload_example: value.payload_example.clone(),
        }
    }
}

// --- credentials -----------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialResponse {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub kind: String,
    pub name: String,
    pub origin: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Credential> for CredentialResponse {
    fn from(value: Credential) -> Self {
        Self {
            id: value.id,
            organization_id: value.org_id,
            kind: value.kind,
            name: value.name,
            origin: value.origin.as_str().to_owned(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// The one response that ever carries a credential's secret — see
/// `credential::create` and `credential::rotate`. `secret` is a JSON object
/// for a `Supplied` credential (exactly what the caller sent) and a string
/// for a `Generated` one (base64 of the freshly minted bytes).
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CredentialWithSecretResponse {
    #[serde(flatten)]
    pub credential: CredentialResponse,
    pub secret: Value,
}

/// Decodes a credential's plaintext into the JSON shape a client should see
/// once — the same decision a worker opening the credential later would
/// make from `Credential::origin`, mirrored here for display rather than
/// use. `Supplied` reconstructs the object the caller originally sent
/// (`create_credential`'s plaintext *is* `serde_json::to_vec(&data)`);
/// `Generated` has no JSON structure of its own, so it is shown as base64.
pub fn secret_value(origin: CredentialOrigin, plaintext: &[u8]) -> Value {
    match origin {
        CredentialOrigin::Supplied => serde_json::from_slice(plaintext).unwrap_or(Value::Null),
        CredentialOrigin::Generated => {
            use base64::Engine;
            Value::String(base64::engine::general_purpose::STANDARD.encode(plaintext))
        }
    }
}

// --- workflows -------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkflowResponse {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub current_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Workflow> for WorkflowResponse {
    fn from(value: Workflow) -> Self {
        Self {
            id: value.id,
            organization_id: value.org_id,
            name: value.name,
            description: value.description,
            enabled: value.enabled,
            current_version_id: value.current_version_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// Mirrors `mestier_core::Branch`'s default `Serialize` shape (its own
/// variant name, bare: `"Then"`, `"Else"`, `"Each"`, `"After"`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum BranchDto {
    Then,
    Else,
    Each,
    After,
}

impl From<mestier_core::Branch> for BranchDto {
    fn from(value: mestier_core::Branch) -> Self {
        match value {
            mestier_core::Branch::Then => Self::Then,
            mestier_core::Branch::Else => Self::Else,
            mestier_core::Branch::Each => Self::Each,
            mestier_core::Branch::After => Self::After,
        }
    }
}

impl From<BranchDto> for mestier_core::Branch {
    fn from(value: BranchDto) -> Self {
        match value {
            BranchDto::Then => Self::Then,
            BranchDto::Else => Self::Else,
            BranchDto::Each => Self::Each,
            BranchDto::After => Self::After,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PlacedConnectorDto {
    pub id: String,
    pub kind: String,
    pub version: u16,
    pub credential_id: Option<Uuid>,
    pub config: serde_json::Map<String, Value>,
}

impl From<mestier_core::PlacedConnector> for PlacedConnectorDto {
    fn from(value: mestier_core::PlacedConnector) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            version: value.version,
            credential_id: value.credential_id,
            config: value.config,
        }
    }
}

impl From<PlacedConnectorDto> for mestier_core::PlacedConnector {
    fn from(value: PlacedConnectorDto) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            version: value.version,
            credential_id: value.credential_id,
            config: value.config,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct EdgeDto {
    pub from: String,
    pub to: String,
    pub branch: Option<BranchDto>,
}

impl From<mestier_core::Edge> for EdgeDto {
    fn from(value: mestier_core::Edge) -> Self {
        Self {
            from: value.from,
            to: value.to,
            branch: value.branch.map(BranchDto::from),
        }
    }
}

impl From<EdgeDto> for mestier_core::Edge {
    fn from(value: EdgeDto) -> Self {
        Self {
            from: value.from,
            to: value.to,
            branch: value.branch.map(mestier_core::Branch::from),
        }
    }
}

/// The workflow graph on the wire — the editor reads and writes this whole,
/// both as the body of `PUT .../versions` and nested in a workflow's
/// current version on `GET .../workflows/{id}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct GraphDto {
    pub connectors: Vec<PlacedConnectorDto>,
    pub edges: Vec<EdgeDto>,
}

impl From<mestier_core::Graph> for GraphDto {
    fn from(value: mestier_core::Graph) -> Self {
        Self {
            connectors: value.connectors.into_iter().map(Into::into).collect(),
            edges: value.edges.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<GraphDto> for mestier_core::Graph {
    fn from(value: GraphDto) -> Self {
        Self {
            connectors: value.connectors.into_iter().map(Into::into).collect(),
            edges: value.edges.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkflowVersionResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub version: i32,
    pub graph: GraphDto,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

impl From<WorkflowVersion> for WorkflowVersionResponse {
    fn from(value: WorkflowVersion) -> Self {
        Self {
            id: value.id,
            workflow_id: value.workflow_id,
            version: value.version,
            graph: GraphDto::from(value.graph),
            created_at: value.created_at,
            created_by: value.created_by,
        }
    }
}

/// A workflow with its current version, when it has one — the shape
/// `GET .../workflows/{id}` returns (#203: "read avec sa version courante").
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkflowDetailResponse {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub current_version: Option<WorkflowVersionResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkflowDetailResponse {
    pub fn new(workflow: Workflow, current_version: Option<WorkflowVersion>) -> Self {
        Self {
            id: workflow.id,
            organization_id: workflow.org_id,
            name: workflow.name,
            description: workflow.description,
            enabled: workflow.enabled,
            current_version: current_version.map(WorkflowVersionResponse::from),
            created_at: workflow.created_at,
            updated_at: workflow.updated_at,
        }
    }
}

/// A structured graph validation error: which connector, and when the
/// mistake is inside one field, which field — see
/// `workflow::save_version::handler` for why this is built by hand from
/// `mestier_core::validate_graph` rather than through `handlers::ApiError`.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct GraphErrorResponse {
    pub connector_id: Option<String>,
    pub field: Option<String>,
    pub message: String,
}

impl From<&mestier_core::GraphError> for GraphErrorResponse {
    fn from(value: &mestier_core::GraphError) -> Self {
        use mestier_core::GraphError::*;

        let (connector_id, field) = match value {
            DuplicateConnectorId { id } => (Some(id.clone()), None),
            UnknownConnectorKind { connector_id, .. } => (Some(connector_id.clone()), None),
            UnknownConfigField {
                connector_id,
                field,
            } => (Some(connector_id.clone()), Some(field.clone())),
            MissingRequiredField {
                connector_id,
                field,
            } => (Some(connector_id.clone()), Some(field.clone())),
            FieldTypeMismatch {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), Some(field.clone())),
            ExpressionNotAllowed {
                connector_id,
                field,
            } => (Some(connector_id.clone()), Some(field.clone())),
            InvalidExpression {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), Some(field.clone())),
            UnknownCredential { connector_id, .. } => (Some(connector_id.clone()), None),
            CredentialSchemeNotAccepted { connector_id, .. } => (Some(connector_id.clone()), None),
            MissingCredential { connector_id } => (Some(connector_id.clone()), None),
            UnknownConnectorReference {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), Some(field.clone())),
            DownstreamConnectorReference {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), Some(field.clone())),
            LoopUsedOutsideLoop {
                connector_id,
                field,
            } => (Some(connector_id.clone()), Some(field.clone())),
            DanglingEdge { from, .. } => (Some(from.clone()), None),
            InvalidBranch { connector_id } => (Some(connector_id.clone()), None),
            Cycle { connector_ids } => (connector_ids.first().cloned(), None),
            UnreachableConnector { connector_id } => (Some(connector_id.clone()), None),
        };

        Self {
            connector_id,
            field,
            message: value.to_string(),
        }
    }
}

// --- runs --------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RunResponse {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub workflow_id: Uuid,
    pub workflow_version_id: Uuid,
    pub trigger_event_id: Option<Uuid>,
    pub trigger_payload: Option<Value>,
    pub status: String,
    pub error: Option<String>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Run> for RunResponse {
    fn from(value: Run) -> Self {
        Self {
            id: value.id,
            organization_id: value.org_id,
            workflow_id: value.workflow_id,
            workflow_version_id: value.workflow_version_id,
            trigger_event_id: value.trigger_event_id,
            trigger_payload: value.trigger_payload,
            status: run_status_str(value.status).to_owned(),
            error: value.error,
            next_attempt_at: value.next_attempt_at,
            started_at: value.started_at,
            finished_at: value.finished_at,
            created_at: value.created_at,
        }
    }
}

fn run_status_str(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Pending => "pending",
        RunStatus::Running => "running",
        RunStatus::Succeeded => "succeeded",
        RunStatus::Failed => "failed",
        RunStatus::Cancelled => "cancelled",
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RunStepResponse {
    pub id: Uuid,
    pub connector_id: String,
    pub iteration_path: String,
    pub attempts: u32,
    pub status: String,
    pub input: Option<Value>,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<RunStep> for RunStepResponse {
    fn from(value: RunStep) -> Self {
        Self {
            id: value.id,
            connector_id: value.connector_id,
            iteration_path: value.iteration_path,
            attempts: value.attempts,
            status: step_status_str(value.status).to_owned(),
            input: value.input,
            output: value.output,
            error: value.error,
            started_at: value.started_at,
            finished_at: value.finished_at,
            created_at: value.created_at,
        }
    }
}

fn step_status_str(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::InFlight => "in_flight",
        StepStatus::Succeeded => "succeeded",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
        StepStatus::Dead => "dead",
    }
}

/// A run with its steps — what `GET .../runs/{id}` returns (#203: "read
/// avec ses pas").
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RunDetailResponse {
    #[serde(flatten)]
    pub run: RunResponse,
    pub steps: Vec<RunStepResponse>,
}

// --- settings ------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AutomationSettingsBody {
    pub event_retention_seconds: u64,
    pub succeeded_run_retention_seconds: u64,
    pub retry_schedule_seconds: Vec<u64>,
    pub disable_target_after: Option<u32>,
}

impl From<AutomationSettings> for AutomationSettingsBody {
    fn from(value: AutomationSettings) -> Self {
        Self {
            event_retention_seconds: value.event_retention.as_secs(),
            succeeded_run_retention_seconds: value.succeeded_run_retention.as_secs(),
            retry_schedule_seconds: value.retry_schedule.iter().map(Duration::as_secs).collect(),
            disable_target_after: value.disable_target_after,
        }
    }
}

impl From<AutomationSettingsBody> for AutomationSettings {
    fn from(value: AutomationSettingsBody) -> Self {
        Self {
            event_retention: Duration::from_secs(value.event_retention_seconds),
            succeeded_run_retention: Duration::from_secs(value.succeeded_run_retention_seconds),
            retry_schedule: value
                .retry_schedule_seconds
                .into_iter()
                .map(Duration::from_secs)
                .collect(),
            disable_target_after: value.disable_target_after,
        }
    }
}

#[cfg(test)]
mod tests {
    use mestier_core::{GraphError, connector_catalogue, event_catalogue};
    use serde_json::json;

    use super::*;

    // --- secret_value --------------------------------------------------

    #[test]
    fn a_supplied_secret_reconstructs_the_original_json_object() {
        let data = json!({ "token": "abc123" });
        let plaintext = serde_json::to_vec(&data).unwrap();

        let secret = secret_value(CredentialOrigin::Supplied, &plaintext);

        assert_eq!(secret, data);
    }

    #[test]
    fn a_generated_secret_is_shown_as_base64() {
        use base64::Engine;
        let plaintext = vec![1u8, 2, 3, 4, 5];

        let secret = secret_value(CredentialOrigin::Generated, &plaintext);

        let expected = base64::engine::general_purpose::STANDARD.encode(&plaintext);
        assert_eq!(secret, Value::String(expected));
    }

    // --- GraphErrorResponse ----------------------------------------------

    #[test]
    fn a_missing_field_error_names_the_connector_and_the_field() {
        let error = GraphError::MissingRequiredField {
            connector_id: "c1".to_string(),
            field: "predicate".to_string(),
        };

        let response = GraphErrorResponse::from(&error);

        assert_eq!(response.connector_id, Some("c1".to_string()));
        assert_eq!(response.field, Some("predicate".to_string()));
        assert!(response.message.contains("c1"));
        assert!(response.message.contains("predicate"));
    }

    /// Not every error is about one field — a missing credential names only
    /// the connector.
    #[test]
    fn a_missing_credential_error_names_only_the_connector() {
        let error = GraphError::MissingCredential {
            connector_id: "c1".to_string(),
        };

        let response = GraphErrorResponse::from(&error);

        assert_eq!(response.connector_id, Some("c1".to_string()));
        assert_eq!(response.field, None);
    }

    #[test]
    fn a_duplicate_connector_id_error_names_the_connector() {
        let error = GraphError::DuplicateConnectorId {
            id: "c1".to_string(),
        };

        let response = GraphErrorResponse::from(&error);

        assert_eq!(response.connector_id, Some("c1".to_string()));
        assert_eq!(response.field, None);
    }

    // --- catalogue DTOs are enough to render a form data-driven --------

    /// The acceptance criterion (#203): the connector catalogue's response
    /// carries enough for the editor to render a form without any
    /// connector known by name in the frontend's own code — proven here by
    /// walking every shipped connector's fields off the *response* type
    /// alone, the same way a frontend would.
    #[test]
    fn every_shipped_connector_is_fully_described_by_its_response() {
        let catalogue = connector_catalogue();
        let responses: Vec<ConnectorDescriptorResponse> = catalogue
            .descriptors()
            .map(ConnectorDescriptorResponse::from)
            .collect();

        assert!(!responses.is_empty());
        for descriptor in &responses {
            assert!(!descriptor.kind.is_empty());
            assert!(!descriptor.label.is_empty());
            assert!(
                !descriptor.output_example.is_null(),
                "`{}` has no output example",
                descriptor.kind
            );
            for field in &descriptor.fields {
                assert!(!field.name.is_empty());
                assert!(!field.label.is_empty());
            }
        }

        let odoo_create_partner = responses
            .iter()
            .find(|d| d.kind == "odoo.create_partner")
            .expect("odoo.create_partner is in the catalogue");
        assert!(
            odoo_create_partner
                .fields
                .iter()
                .any(|f| f.name == "name" && f.required)
        );
        assert_eq!(
            odoo_create_partner.auth,
            AuthRequirementResponse::Exactly("odoo_api".to_string())
        );
    }

    #[test]
    fn every_auth_scheme_referenced_by_a_connector_is_in_the_response() {
        use mestier_core::auth_schemes;

        let catalogue = connector_catalogue();
        let scheme_responses: Vec<AuthSchemeResponse> = auth_schemes()
            .iter()
            .map(AuthSchemeResponse::from)
            .collect();
        let known_kinds: std::collections::HashSet<&str> =
            scheme_responses.iter().map(|s| s.kind.as_str()).collect();

        for descriptor in catalogue.descriptors() {
            for scheme_kind in descriptor.auth.scheme_kinds() {
                assert!(
                    known_kinds.contains(scheme_kind),
                    "`{}` requires scheme `{scheme_kind}`, absent from the response",
                    descriptor.kind
                );
            }
        }
    }

    #[test]
    fn the_event_catalogue_response_carries_a_payload_example_for_every_event() {
        let catalogue = event_catalogue();

        for descriptor in catalogue.descriptors() {
            let response = EventDescriptorResponse::from(descriptor);
            assert!(!response.name.is_empty());
            assert!(
                !response.payload_example.is_null(),
                "`{}` has no payload example",
                response.name
            );
        }
    }

    // --- settings round trip --------------------------------------------

    #[test]
    fn automation_settings_round_trips_through_its_wire_body() {
        let settings = AutomationSettings {
            event_retention: Duration::from_secs(1000),
            succeeded_run_retention: Duration::from_secs(2000),
            retry_schedule: vec![Duration::from_secs(5), Duration::from_secs(30)],
            disable_target_after: Some(7),
        };

        let body = AutomationSettingsBody::from(settings.clone());
        let round_tripped: AutomationSettings = body.into();

        assert_eq!(round_tripped, settings);
    }
}
