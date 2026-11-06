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

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mestier_core::{
    AuthRequirement, AutomationSettings, CredentialFieldKind, CredentialKind, Field, FieldKind,
    NodePosition, OrganizationId, RunOverview, RunStatus, RunStep, SelectOption, StepStatus,
    VisibleWhen, Workflow, WorkflowFolder, WorkflowVersion,
};
use serde::{Deserialize, Deserializer, Serialize, de};
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
    Optional(Vec<String>),
}

impl From<&AuthRequirement> for AuthRequirementResponse {
    fn from(value: &AuthRequirement) -> Self {
        match value {
            AuthRequirement::None => Self::None,
            AuthRequirement::Exactly(kind) => Self::Exactly((*kind).to_owned()),
            AuthRequirement::AnyOf(kinds) => {
                Self::AnyOf(kinds.iter().map(|k| (*k).to_owned()).collect())
            }
            AuthRequirement::Optional(kinds) => {
                Self::Optional(kinds.iter().map(|k| (*k).to_owned()).collect())
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
    pub branches: Vec<BranchDto>,
    pub execution: ExecutionModeResponse,
    pub output_example: Value,
    pub output_mirrors_field: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionModeResponse {
    PerItem,
    Once,
}

impl From<mestier_core::ExecutionMode> for ExecutionModeResponse {
    fn from(value: mestier_core::ExecutionMode) -> Self {
        match value {
            mestier_core::ExecutionMode::PerItem => Self::PerItem,
            mestier_core::ExecutionMode::Once => Self::Once,
        }
    }
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
            branches: value
                .branches
                .iter()
                .copied()
                .map(BranchDto::from)
                .collect(),
            execution: value.execution.into(),
            output_example: value.output_example.clone(),
            output_mirrors_field: value.output_mirrors_field.map(str::to_owned),
            group: value.group.map(str::to_owned),
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

impl From<&CredentialKind> for AuthSchemeResponse {
    fn from(value: &CredentialKind) -> Self {
        Self {
            kind: value.kind.to_owned(),
            label: value.label.to_owned(),
            fields: value
                .fields
                .iter()
                .map(|field| FieldResponse {
                    name: field.name.to_owned(),
                    label: field.label.to_owned(),
                    required: field.required,
                    kind: match field.kind {
                        CredentialFieldKind::Text
                        | CredentialFieldKind::Password
                        | CredentialFieldKind::Textarea => FieldKindResponse::Text,
                        CredentialFieldKind::Number => FieldKindResponse::Number,
                        CredentialFieldKind::Boolean => FieldKindResponse::Bool,
                        CredentialFieldKind::Json => FieldKindResponse::Json,
                        CredentialFieldKind::Select => FieldKindResponse::Select {
                            options: field
                                .options
                                .iter()
                                .map(|option| SelectOptionResponse {
                                    value: option.value.to_owned(),
                                    label: option.label.to_owned(),
                                })
                                .collect(),
                        },
                    },
                    expression: false,
                    secret: field.secret,
                    visible_when: field.show_when.map(|rule| VisibleWhenResponse {
                        field: rule.field.to_owned(),
                        any_of: rule
                            .one_of
                            .iter()
                            .map(|value| (*value).to_owned())
                            .collect(),
                    }),
                })
                .collect(),
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

pub use crate::credential::dto::{
    CredentialKindResponse, CredentialResponse, CredentialWithSecretResponse, secret_value,
};

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct FolderResponse {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<WorkflowFolder> for FolderResponse {
    fn from(value: WorkflowFolder) -> Self {
        Self {
            id: value.id,
            name: value.name.into_string(),
            parent_id: value.parent_id,
            created_at: value.created_at,
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
    pub folder_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub archived_at: Option<DateTime<Utc>>,
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
            folder_id: value.folder_id,
            tags: value.tags.into_vec(),
            archived_at: value.archived_at,
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
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub operation: Option<OperationRefDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OperationRefDto {
    pub collection_id: Uuid,
    pub operation_id: String,
}

impl From<mestier_core::OperationRef> for OperationRefDto {
    fn from(value: mestier_core::OperationRef) -> Self {
        Self {
            collection_id: value.collection_id,
            operation_id: value.operation_id,
        }
    }
}

impl From<OperationRefDto> for mestier_core::OperationRef {
    fn from(value: OperationRefDto) -> Self {
        Self {
            collection_id: value.collection_id,
            operation_id: value.operation_id,
        }
    }
}

impl From<mestier_core::PlacedConnector> for PlacedConnectorDto {
    fn from(value: mestier_core::PlacedConnector) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            version: value.version,
            credential_id: value.credential_id,
            config: value.config,
            name: value.name,
            description: value.description,
            disabled: value.disabled,
            operation: value.operation.map(Into::into),
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
            name: value.name,
            description: value.description,
            disabled: value.disabled,
            operation: value.operation.map(Into::into),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum TriggerKindDto {
    Manual,
    Events(Vec<String>),
    Webhook {
        method: WebhookMethodDto,
        path: String,
        #[serde(default)]
        credential_id: Option<Uuid>,
        #[serde(default)]
        contract: Vec<ContractKeyDto>,
        #[serde(default)]
        on_invalid: OnInvalidDto,
    },
    Schedule {
        frequency: FrequencyDto,
        time: String,
        #[serde(default)]
        weekday: Option<WeekdayDto>,
        #[serde(default)]
        timezone: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum WebhookMethodDto {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum ContractKindDto {
    Text,
    Number,
    Boolean,
    Object,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ContractKeyDto {
    pub name: String,
    pub kind: ContractKindDto,
    pub required: bool,
    #[serde(default)]
    pub example: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum OnInvalidDto {
    #[default]
    Reject,
    Accept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum FrequencyDto {
    Hourly,
    Daily,
    Weekly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum WeekdayDto {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<mestier_core::WebhookMethod> for WebhookMethodDto {
    fn from(value: mestier_core::WebhookMethod) -> Self {
        match value {
            mestier_core::WebhookMethod::Get => Self::Get,
            mestier_core::WebhookMethod::Post => Self::Post,
            mestier_core::WebhookMethod::Put => Self::Put,
            mestier_core::WebhookMethod::Patch => Self::Patch,
            mestier_core::WebhookMethod::Delete => Self::Delete,
        }
    }
}

impl From<WebhookMethodDto> for mestier_core::WebhookMethod {
    fn from(value: WebhookMethodDto) -> Self {
        match value {
            WebhookMethodDto::Get => Self::Get,
            WebhookMethodDto::Post => Self::Post,
            WebhookMethodDto::Put => Self::Put,
            WebhookMethodDto::Patch => Self::Patch,
            WebhookMethodDto::Delete => Self::Delete,
        }
    }
}

impl From<mestier_core::ContractKind> for ContractKindDto {
    fn from(value: mestier_core::ContractKind) -> Self {
        match value {
            mestier_core::ContractKind::Text => Self::Text,
            mestier_core::ContractKind::Number => Self::Number,
            mestier_core::ContractKind::Boolean => Self::Boolean,
            mestier_core::ContractKind::Object => Self::Object,
            mestier_core::ContractKind::List => Self::List,
        }
    }
}

impl From<ContractKindDto> for mestier_core::ContractKind {
    fn from(value: ContractKindDto) -> Self {
        match value {
            ContractKindDto::Text => Self::Text,
            ContractKindDto::Number => Self::Number,
            ContractKindDto::Boolean => Self::Boolean,
            ContractKindDto::Object => Self::Object,
            ContractKindDto::List => Self::List,
        }
    }
}

impl From<mestier_core::ContractKey> for ContractKeyDto {
    fn from(value: mestier_core::ContractKey) -> Self {
        Self {
            name: value.name,
            kind: value.kind.into(),
            required: value.required,
            example: value.example,
        }
    }
}

impl From<ContractKeyDto> for mestier_core::ContractKey {
    fn from(value: ContractKeyDto) -> Self {
        Self {
            name: value.name,
            kind: value.kind.into(),
            required: value.required,
            example: value.example,
        }
    }
}

impl From<mestier_core::OnInvalid> for OnInvalidDto {
    fn from(value: mestier_core::OnInvalid) -> Self {
        match value {
            mestier_core::OnInvalid::Reject => Self::Reject,
            mestier_core::OnInvalid::Accept => Self::Accept,
        }
    }
}

impl From<OnInvalidDto> for mestier_core::OnInvalid {
    fn from(value: OnInvalidDto) -> Self {
        match value {
            OnInvalidDto::Reject => Self::Reject,
            OnInvalidDto::Accept => Self::Accept,
        }
    }
}

impl From<mestier_core::Frequency> for FrequencyDto {
    fn from(value: mestier_core::Frequency) -> Self {
        match value {
            mestier_core::Frequency::Hourly => Self::Hourly,
            mestier_core::Frequency::Daily => Self::Daily,
            mestier_core::Frequency::Weekly => Self::Weekly,
        }
    }
}

impl From<FrequencyDto> for mestier_core::Frequency {
    fn from(value: FrequencyDto) -> Self {
        match value {
            FrequencyDto::Hourly => Self::Hourly,
            FrequencyDto::Daily => Self::Daily,
            FrequencyDto::Weekly => Self::Weekly,
        }
    }
}

impl From<mestier_core::Weekday> for WeekdayDto {
    fn from(value: mestier_core::Weekday) -> Self {
        match value {
            mestier_core::Weekday::Monday => Self::Monday,
            mestier_core::Weekday::Tuesday => Self::Tuesday,
            mestier_core::Weekday::Wednesday => Self::Wednesday,
            mestier_core::Weekday::Thursday => Self::Thursday,
            mestier_core::Weekday::Friday => Self::Friday,
            mestier_core::Weekday::Saturday => Self::Saturday,
            mestier_core::Weekday::Sunday => Self::Sunday,
        }
    }
}

impl From<WeekdayDto> for mestier_core::Weekday {
    fn from(value: WeekdayDto) -> Self {
        match value {
            WeekdayDto::Monday => Self::Monday,
            WeekdayDto::Tuesday => Self::Tuesday,
            WeekdayDto::Wednesday => Self::Wednesday,
            WeekdayDto::Thursday => Self::Thursday,
            WeekdayDto::Friday => Self::Friday,
            WeekdayDto::Saturday => Self::Saturday,
            WeekdayDto::Sunday => Self::Sunday,
        }
    }
}

impl From<mestier_core::TriggerKind> for TriggerKindDto {
    fn from(value: mestier_core::TriggerKind) -> Self {
        match value {
            mestier_core::TriggerKind::Manual => Self::Manual,
            mestier_core::TriggerKind::Events(names) => Self::Events(names),
            mestier_core::TriggerKind::Webhook {
                method,
                path,
                credential_id,
                contract,
                on_invalid,
            } => Self::Webhook {
                method: method.into(),
                path,
                credential_id,
                contract: contract.into_iter().map(Into::into).collect(),
                on_invalid: on_invalid.into(),
            },
            mestier_core::TriggerKind::Schedule {
                frequency,
                time,
                weekday,
                timezone,
            } => Self::Schedule {
                frequency: frequency.into(),
                time,
                weekday: weekday.map(Into::into),
                timezone,
            },
        }
    }
}

impl From<TriggerKindDto> for mestier_core::TriggerKind {
    fn from(value: TriggerKindDto) -> Self {
        match value {
            TriggerKindDto::Manual => Self::Manual,
            TriggerKindDto::Events(names) => Self::Events(names),
            TriggerKindDto::Webhook {
                method,
                path,
                credential_id,
                contract,
                on_invalid,
            } => Self::Webhook {
                method: method.into(),
                path,
                credential_id,
                contract: contract.into_iter().map(Into::into).collect(),
                on_invalid: on_invalid.into(),
            },
            TriggerKindDto::Schedule {
                frequency,
                time,
                weekday,
                timezone,
            } => Self::Schedule {
                frequency: frequency.into(),
                time,
                weekday: weekday.map(Into::into),
                timezone,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PlacedTriggerDto {
    pub id: String,
    pub kind: TriggerKindDto,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl From<mestier_core::PlacedTrigger> for PlacedTriggerDto {
    fn from(value: mestier_core::PlacedTrigger) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            name: value.name,
            description: value.description,
        }
    }
}

impl From<PlacedTriggerDto> for mestier_core::PlacedTrigger {
    fn from(value: PlacedTriggerDto) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            name: value.name,
            description: value.description,
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
    pub triggers: Vec<PlacedTriggerDto>,
}

impl From<mestier_core::Graph> for GraphDto {
    fn from(value: mestier_core::Graph) -> Self {
        Self {
            connectors: value.connectors.into_iter().map(Into::into).collect(),
            edges: value.edges.into_iter().map(Into::into).collect(),
            triggers: value.triggers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<GraphDto> for mestier_core::Graph {
    fn from(value: GraphDto) -> Self {
        Self {
            connectors: value.connectors.into_iter().map(Into::into).collect(),
            edges: value.edges.into_iter().map(Into::into).collect(),
            triggers: value.triggers.into_iter().map(Into::into).collect(),
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct NodePositionDto {
    pub x: f64,
    pub y: f64,
}

impl From<NodePosition> for NodePositionDto {
    fn from(value: NodePosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<NodePositionDto> for NodePosition {
    fn from(value: NodePositionDto) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[schema(value_type = std::collections::BTreeMap<String, NodePositionDto>)]
#[serde(transparent)]
pub struct WorkflowLayoutDto(BTreeMap<String, NodePositionDto>);

impl<'de> Deserialize<'de> for WorkflowLayoutDto {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let positions = BTreeMap::<String, NodePositionDto>::deserialize(deserializer)?;

        if positions.len() > mestier_core::MAX_LAYOUT_ENTRIES {
            return Err(de::Error::custom(format!(
                "a layout holds at most {} entries, got {}",
                mestier_core::MAX_LAYOUT_ENTRIES,
                positions.len()
            )));
        }

        Ok(Self(positions))
    }
}

impl From<mestier_core::WorkflowLayout> for WorkflowLayoutDto {
    fn from(value: mestier_core::WorkflowLayout) -> Self {
        let positions: BTreeMap<String, NodePosition> = value.into();
        Self(
            positions
                .into_iter()
                .map(|(id, position)| (id, position.into()))
                .collect(),
        )
    }
}

impl From<WorkflowLayoutDto> for mestier_core::WorkflowLayout {
    fn from(value: WorkflowLayoutDto) -> Self {
        value
            .0
            .into_iter()
            .map(|(id, position)| (id, position.into()))
            .collect::<BTreeMap<_, _>>()
            .into()
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
    pub folder_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub layout: Option<WorkflowLayoutDto>,
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
            folder_id: workflow.folder_id,
            tags: workflow.tags.into_vec(),
            archived_at: workflow.archived_at,
            layout: workflow.layout.map(WorkflowLayoutDto::from),
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
    pub trigger_id: Option<String>,
    pub field: Option<String>,
    pub message: String,
}

impl GraphErrorResponse {
    pub fn quota(message: &str) -> Self {
        Self {
            connector_id: None,
            trigger_id: None,
            field: None,
            message: message.to_owned(),
        }
    }

    pub fn webhook_path_taken(trigger_id: String) -> Self {
        Self {
            connector_id: None,
            message: format!(
                "trigger `{trigger_id}` uses a webhook path another workflow already holds"
            ),
            trigger_id: Some(trigger_id),
            field: Some("path".to_owned()),
        }
    }
}

fn node_fault(
    node: &mestier_core::NodeRef,
    field: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let id = node.id().to_owned();
    match node {
        mestier_core::NodeRef::Connector(_) => (Some(id), None, Some(field.to_owned())),
        mestier_core::NodeRef::Trigger(_) => (None, Some(id), Some(field.to_owned())),
    }
}

impl From<&mestier_core::GraphError> for GraphErrorResponse {
    fn from(value: &mestier_core::GraphError) -> Self {
        use mestier_core::GraphError::*;

        let (connector_id, trigger_id, field) = match value {
            DuplicateConnectorId { id } => (Some(id.clone()), None, None),
            UnknownConnectorKind { connector_id, .. } => (Some(connector_id.clone()), None, None),
            UnknownConfigField {
                connector_id,
                field,
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            MissingRequiredField {
                connector_id,
                field,
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            FieldTypeMismatch {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            ExpressionNotAllowed {
                connector_id,
                field,
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            InvalidExpression {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            UnknownCollection { connector_id, .. } | UnknownOperation { connector_id, .. } => (
                Some(connector_id.clone()),
                None,
                Some("operation".to_owned()),
            ),
            MissingParameter {
                connector_id,
                field,
            }
            | UnknownParameter {
                connector_id,
                field,
            }
            | InvalidParameter {
                connector_id,
                field,
            }
            | DuplicateParameter {
                connector_id,
                field,
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            UnknownCredential { connector_id, .. } => (Some(connector_id.clone()), None, None),
            CredentialSchemeNotAccepted { connector_id, .. } => {
                (Some(connector_id.clone()), None, None)
            }
            MissingCredential { connector_id } => (Some(connector_id.clone()), None, None),
            HeaderInjection {
                connector_id,
                field,
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            TooManyRecipients { connector_id } => {
                (Some(connector_id.clone()), None, Some("to".to_owned()))
            }
            UnknownNodeReference {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            NodeNotUpstream {
                connector_id,
                field,
                ..
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            LoopUsedOutsideLoop {
                connector_id,
                field,
            } => (Some(connector_id.clone()), None, Some(field.clone())),
            DanglingEdge { from, .. } => (Some(from.clone()), None, None),
            InvalidBranch { connector_id } => (Some(connector_id.clone()), None, None),
            Cycle { connector_ids } => (connector_ids.first().cloned(), None, None),
            UnreachableConnector { connector_id } => (Some(connector_id.clone()), None, None),
            DuplicateTriggerId { id } => (None, Some(id.clone()), None),
            TriggerIdCollidesWithConnectorId { id } => (None, Some(id.clone()), None),
            EdgeIntoTrigger { trigger_id, .. } => (None, Some(trigger_id.clone()), None),
            BranchedTriggerEdge { trigger_id, .. } => (None, Some(trigger_id.clone()), None),
            InertTrigger { trigger_id } => (None, Some(trigger_id.clone()), None),
            EmptyEventTrigger { trigger_id } => (None, Some(trigger_id.clone()), None),
            UnknownEventName { trigger_id, .. } => (None, Some(trigger_id.clone()), None),
            EmptyNodeName { node } | NodeNameTooLong { node } | DuplicateNodeName { node, .. } => {
                node_fault(node, "name")
            }
            NodeDescriptionTooLong { node } => node_fault(node, "description"),
            InvalidWebhookPath { trigger_id, .. } | DuplicateWebhookPath { trigger_id, .. } => {
                (None, Some(trigger_id.clone()), Some("path".to_owned()))
            }
            UnknownWebhookCredential { trigger_id, .. }
            | WebhookCredentialSchemeNotAccepted { trigger_id, .. } => (
                None,
                Some(trigger_id.clone()),
                Some("credential".to_owned()),
            ),
            EmptyContractKeyName { trigger_id } => {
                (None, Some(trigger_id.clone()), Some("contract".to_owned()))
            }
            DuplicateContractKey { trigger_id, name } => (
                None,
                Some(trigger_id.clone()),
                Some(format!("contract.{name}")),
            ),
            InvalidScheduleTime { trigger_id, .. } => {
                (None, Some(trigger_id.clone()), Some("time".to_owned()))
            }
            InvalidContractKeyName { trigger_id, name } => (
                None,
                Some(trigger_id.clone()),
                Some(format!("contract.{name}")),
            ),
            TooManyTriggers => (None, None, Some("triggers".to_owned())),
            UnknownScheduleTimezone { trigger_id, .. } => {
                (None, Some(trigger_id.clone()), Some("timezone".to_owned()))
            }
        };

        Self {
            connector_id,
            trigger_id,
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
    pub number: i64,
    pub steps_done: i32,
    pub steps_total: i32,
    pub trigger_id: Option<String>,
    pub trigger_event_name: Option<String>,
    pub failed_connector_id: Option<String>,
    pub workflow_version_created_at: DateTime<Utc>,
}

impl From<RunOverview> for RunResponse {
    fn from(value: RunOverview) -> Self {
        let run = value.run;
        Self {
            id: run.id,
            organization_id: run.org_id,
            workflow_id: run.workflow_id,
            workflow_version_id: run.workflow_version_id,
            trigger_event_id: run.trigger_event_id,
            trigger_payload: run.trigger_payload,
            status: run_status_str(run.status).to_owned(),
            error: run.error,
            next_attempt_at: run.next_attempt_at,
            started_at: run.started_at,
            finished_at: run.finished_at,
            created_at: run.created_at,
            number: value.number,
            steps_done: value.steps_done,
            steps_total: value.steps_total,
            trigger_id: run.trigger_id,
            trigger_event_name: value.trigger_event_name,
            failed_connector_id: value.failed_connector_id,
            workflow_version_created_at: value.workflow_version_created_at,
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
    pub resume_at: Option<DateTime<Utc>>,
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
            resume_at: value.resume_at,
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
        StepStatus::Waiting => "waiting",
    }
}

/// A run with its steps — what `GET .../runs/{id}` returns (#203: "read
/// avec ses pas").
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RunDetailResponse {
    #[serde(flatten)]
    pub run: RunResponse,
    pub steps: Vec<RunStepResponse>,
    pub graph: Option<GraphDto>,
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
    use mestier_core::{GraphError, Run, connector_catalogue, event_catalogue};
    use serde_json::json;

    use super::*;

    #[test]
    fn a_graph_crossing_the_wire_keeps_its_triggers() {
        let domain = mestier_core::Graph {
            connectors: vec![mestier_core::PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config: serde_json::Map::new(),
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }],
            edges: vec![mestier_core::Edge {
                from: "t1".to_string(),
                to: "c1".to_string(),
                branch: None,
            }],
            triggers: vec![mestier_core::PlacedTrigger {
                id: "t1".to_string(),
                kind: mestier_core::TriggerKind::Manual,
                name: None,
                description: None,
            }],
        };

        let round_tripped: mestier_core::Graph = GraphDto::from(domain.clone()).into();

        assert_eq!(round_tripped, domain);
    }

    #[test]
    fn a_graph_the_editor_sends_is_still_saveable() {
        let sent = serde_json::from_value::<GraphDto>(json!({
            "connectors": [{
                "id": "c1",
                "kind": "flow.condition",
                "version": 1,
                "credential_id": null,
                "config": { "predicate": "{{ true }}" }
            }],
            "edges": [{ "from": "t1", "to": "c1", "branch": null }],
            "triggers": [{ "id": "t1", "kind": "Manual" }]
        }))
        .expect("the editor's graph deserializes");

        let graph: mestier_core::Graph = sent.into();

        assert_eq!(
            mestier_core::validate_graph(
                &graph,
                &connector_catalogue(),
                &[],
                &event_catalogue(),
                &[]
            ),
            Ok(())
        );
    }

    #[test]
    fn a_node_carries_its_operation_reference_across_the_wire_and_back() {
        let collection_id = Uuid::from_u128(77);
        let sent = serde_json::from_value::<GraphDto>(json!({
            "connectors": [{
                "id": "c1",
                "kind": "collection.operation",
                "version": 1,
                "credential_id": null,
                "config": { "path": { "id": "42" } },
                "operation": {
                    "collection_id": "00000000-0000-0000-0000-00000000004d",
                    "operation_id": "getAddress"
                }
            }],
            "edges": [{ "from": "t1", "to": "c1", "branch": null }],
            "triggers": [{ "id": "t1", "kind": "Manual" }]
        }))
        .expect("the editor's graph deserializes");

        let domain: mestier_core::Graph = sent.clone().into();
        let operation = domain.connectors[0]
            .operation
            .as_ref()
            .expect("the reference reaches the domain");
        assert_eq!(operation.collection_id, collection_id);
        assert_eq!(operation.operation_id, "getAddress");

        let back = serde_json::to_value(GraphDto::from(domain)).unwrap();
        assert_eq!(
            back["connectors"][0]["operation"],
            json!({
                "collection_id": "00000000-0000-0000-0000-00000000004d",
                "operation_id": "getAddress"
            })
        );
        assert_eq!(
            GraphDto::from(mestier_core::Graph::from(sent.clone())),
            sent
        );
    }

    #[test]
    fn a_node_sent_without_an_operation_reads_as_none_and_is_answered_with_null() {
        let sent = serde_json::from_value::<GraphDto>(json!({
            "connectors": [{
                "id": "c1",
                "kind": "flow.condition",
                "version": 1,
                "credential_id": null,
                "config": { "predicate": "{{ true }}" }
            }],
            "edges": [],
            "triggers": []
        }))
        .expect("an old payload deserializes");

        assert_eq!(sent.connectors[0].operation, None);
        let answered = serde_json::to_value(&sent).unwrap();
        assert!(answered["connectors"][0]["operation"].is_null());
    }

    #[test]
    fn a_graph_omitting_its_triggers_is_refused_on_the_wire() {
        let parsed = serde_json::from_value::<GraphDto>(json!({
            "connectors": [],
            "edges": []
        }));

        assert!(parsed.is_err());
    }

    // --- GraphErrorResponse ----------------------------------------------

    #[test]
    fn collection_errors_name_the_node_and_the_field_at_fault() {
        let collection_id = Uuid::from_u128(9);
        let cases = [
            (
                GraphError::UnknownCollection {
                    connector_id: "c1".to_string(),
                    collection_id,
                },
                "operation",
            ),
            (
                GraphError::UnknownOperation {
                    connector_id: "c1".to_string(),
                    collection_id,
                    operation_id: "getAddress".to_string(),
                },
                "operation",
            ),
            (
                GraphError::MissingParameter {
                    connector_id: "c1".to_string(),
                    field: "path.id".to_string(),
                },
                "path.id",
            ),
            (
                GraphError::UnknownParameter {
                    connector_id: "c1".to_string(),
                    field: "query.sort".to_string(),
                },
                "query.sort",
            ),
        ];

        for (error, field) in cases {
            let response = GraphErrorResponse::from(&error);

            assert_eq!(response.connector_id, Some("c1".to_string()), "{error}");
            assert_eq!(response.trigger_id, None, "{error}");
            assert_eq!(response.field, Some(field.to_string()), "{error}");
            assert!(response.message.contains("c1"), "{}", response.message);
        }
    }

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

    #[test]
    fn a_node_reference_error_names_the_connector_the_field_and_the_node() {
        let unknown = GraphErrorResponse::from(&GraphError::UnknownNodeReference {
            connector_id: "c2".to_string(),
            field: "url".to_string(),
            node_name: "Rechercher les devis".to_string(),
        });
        let downstream = GraphErrorResponse::from(&GraphError::NodeNotUpstream {
            connector_id: "c2".to_string(),
            field: "body.items.0.name".to_string(),
            node_name: "Créer un projet".to_string(),
        });

        assert_eq!(unknown.connector_id, Some("c2".to_string()));
        assert_eq!(unknown.field, Some("url".to_string()));
        assert!(unknown.message.contains("Rechercher les devis"));
        assert_eq!(downstream.connector_id, Some("c2".to_string()));
        assert_eq!(downstream.field, Some("body.items.0.name".to_string()));
        assert!(downstream.message.contains("Créer un projet"));
    }

    #[test]
    fn a_descriptor_response_carries_how_the_connector_runs_over_items() {
        let catalogue = connector_catalogue();

        let once = ConnectorDescriptorResponse::from(catalogue.get("flow.merge", 1).unwrap());
        let per_item = ConnectorDescriptorResponse::from(catalogue.get("smtp.send", 1).unwrap());

        assert_eq!(once.execution, ExecutionModeResponse::Once);
        assert_eq!(per_item.execution, ExecutionModeResponse::PerItem);
        assert_eq!(
            serde_json::to_value(once.execution).unwrap(),
            serde_json::json!("once")
        );
        assert_eq!(
            serde_json::to_value(per_item.execution).unwrap(),
            serde_json::json!("per_item")
        );
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

        let branches_of = |kind: &str| {
            responses
                .iter()
                .find(|d| d.kind == kind)
                .unwrap_or_else(|| panic!("`{kind}` is in the catalogue"))
                .branches
                .clone()
        };
        assert_eq!(
            branches_of("flow.condition"),
            vec![BranchDto::Then, BranchDto::Else]
        );
        assert_eq!(
            branches_of("flow.loop"),
            vec![BranchDto::Each, BranchDto::After]
        );
        assert!(branches_of("http.request").is_empty());

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
        use mestier_core::credential_kinds;

        let catalogue = connector_catalogue();
        let scheme_responses: Vec<AuthSchemeResponse> = credential_kinds()
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

    #[test]
    fn a_layout_dto_round_trips_through_its_wire_shape() {
        let layout: mestier_core::WorkflowLayout = serde_json::from_value(json!({
            "c1": { "x": 10.5, "y": -20.0 },
        }))
        .unwrap();

        let dto = WorkflowLayoutDto::from(layout.clone());

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({ "c1": { "x": 10.5, "y": -20.0 } })
        );
        assert_eq!(mestier_core::WorkflowLayout::from(dto), layout);
    }

    #[test]
    fn a_layout_dto_holding_more_entries_than_the_cap_is_refused() {
        let mut oversized = serde_json::Map::new();
        for index in 0..=mestier_core::MAX_LAYOUT_ENTRIES {
            oversized.insert(format!("c{index}"), json!({ "x": 0.0, "y": 0.0 }));
        }

        let parsed: Result<WorkflowLayoutDto, _> =
            serde_json::from_value(serde_json::Value::Object(oversized));

        assert!(parsed.is_err());
    }

    #[test]
    fn a_node_dto_without_the_new_fields_reads_with_their_defaults() {
        let connector: PlacedConnectorDto = serde_json::from_value(json!({
            "id": "c1",
            "kind": "flow.condition",
            "version": 1,
            "credential_id": null,
            "config": {}
        }))
        .expect("an older client payload still parses");
        let trigger: PlacedTriggerDto = serde_json::from_value(json!({
            "id": "t1",
            "kind": "Manual"
        }))
        .expect("an older client payload still parses");

        assert_eq!(connector.name, None);
        assert_eq!(connector.description, None);
        assert!(!connector.disabled);
        assert_eq!(trigger.name, None);
        assert_eq!(trigger.description, None);
    }

    #[test]
    fn node_labels_and_the_disabled_flag_cross_the_wire_both_ways() {
        let dto: GraphDto = serde_json::from_value(json!({
            "connectors": [{
                "id": "c1",
                "kind": "flow.condition",
                "version": 1,
                "credential_id": null,
                "config": {},
                "name": "Is it paid",
                "description": "Stops unpaid quotes",
                "disabled": true
            }],
            "edges": [{ "from": "t1", "to": "c1", "branch": null }],
            "triggers": [{
                "id": "t1",
                "kind": "Manual",
                "name": "Start",
                "description": "By hand"
            }]
        }))
        .expect("parses");

        let domain = mestier_core::Graph::from(dto.clone());
        assert_eq!(domain.connectors[0].name.as_deref(), Some("Is it paid"));
        assert_eq!(
            domain.connectors[0].description.as_deref(),
            Some("Stops unpaid quotes")
        );
        assert!(domain.connectors[0].disabled);
        assert_eq!(domain.triggers[0].name.as_deref(), Some("Start"));
        assert_eq!(domain.triggers[0].description.as_deref(), Some("By hand"));

        let back = serde_json::to_value(GraphDto::from(domain)).unwrap();
        assert_eq!(back["connectors"][0]["name"], "Is it paid");
        assert_eq!(back["connectors"][0]["description"], "Stops unpaid quotes");
        assert_eq!(back["connectors"][0]["disabled"], true);
        assert_eq!(back["triggers"][0]["name"], "Start");
        assert_eq!(back["triggers"][0]["description"], "By hand");
        assert_eq!(GraphDto::from(mestier_core::Graph::from(dto.clone())), dto);
    }

    #[test]
    fn a_graph_dto_always_serializes_the_node_fields() {
        let dto = GraphDto::from(mestier_core::Graph {
            connectors: vec![mestier_core::PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config: serde_json::Map::new(),
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }],
            edges: Vec::new(),
            triggers: vec![mestier_core::PlacedTrigger {
                id: "t1".to_string(),
                kind: mestier_core::TriggerKind::Manual,
                name: None,
                description: None,
            }],
        });

        let json = serde_json::to_value(dto).unwrap();

        assert_eq!(json["connectors"][0]["name"], serde_json::Value::Null);
        assert_eq!(
            json["connectors"][0]["description"],
            serde_json::Value::Null
        );
        assert_eq!(json["connectors"][0]["disabled"], false);
        assert_eq!(json["triggers"][0]["name"], serde_json::Value::Null);
        assert_eq!(json["triggers"][0]["description"], serde_json::Value::Null);
    }

    #[test]
    fn node_label_errors_name_the_node_and_the_field_at_fault() {
        let on_connector = GraphErrorResponse::from(&GraphError::NodeNameTooLong {
            node: mestier_core::NodeRef::Connector("c1".to_string()),
        });
        let on_trigger = GraphErrorResponse::from(&GraphError::NodeDescriptionTooLong {
            node: mestier_core::NodeRef::Trigger("t1".to_string()),
        });
        let duplicate = GraphErrorResponse::from(&GraphError::DuplicateNodeName {
            node: mestier_core::NodeRef::Connector("c2".to_string()),
            name: "Is it paid".to_string(),
        });
        let empty = GraphErrorResponse::from(&GraphError::EmptyNodeName {
            node: mestier_core::NodeRef::Trigger("t1".to_string()),
        });

        assert_eq!(on_connector.connector_id.as_deref(), Some("c1"));
        assert_eq!(on_connector.trigger_id, None);
        assert_eq!(on_connector.field.as_deref(), Some("name"));
        assert_eq!(on_trigger.connector_id, None);
        assert_eq!(on_trigger.trigger_id.as_deref(), Some("t1"));
        assert_eq!(on_trigger.field.as_deref(), Some("description"));
        assert_eq!(duplicate.connector_id.as_deref(), Some("c2"));
        assert_eq!(duplicate.field.as_deref(), Some("name"));
        assert!(duplicate.message.contains("Is it paid"));
        assert_eq!(empty.trigger_id.as_deref(), Some("t1"));
        assert_eq!(empty.field.as_deref(), Some("name"));
    }

    fn step(status: StepStatus, resume_at: Option<DateTime<Utc>>) -> RunStep {
        let instant = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("valid instant");
        RunStep {
            id: Uuid::from_u128(7),
            run_id: Uuid::from_u128(1),
            connector_id: "w1".to_string(),
            iteration_path: String::new(),
            attempts: 1,
            status,
            input: None,
            output: Some(serde_json::json!({ "duration": 2, "unit": "days" })),
            error: None,
            resume_at,
            started_at: Some(instant),
            finished_at: None,
            created_at: instant,
        }
    }

    #[test]
    fn a_parked_step_is_reported_as_waiting_with_its_resume_time() {
        let resume_at = DateTime::<Utc>::from_timestamp(1_700_172_800, 0).expect("valid instant");

        let json = serde_json::to_value(RunStepResponse::from(step(
            StepStatus::Waiting,
            Some(resume_at),
        )))
        .unwrap();

        assert_eq!(json["status"], "waiting");
        assert_eq!(json["resume_at"], "2023-11-16T22:13:20Z");
        assert_eq!(json["connector_id"], "w1");
    }

    #[test]
    fn a_step_that_is_not_parked_has_no_resume_time() {
        let json =
            serde_json::to_value(RunStepResponse::from(step(StepStatus::Succeeded, None))).unwrap();

        assert_eq!(json["status"], "succeeded");
        assert!(json["resume_at"].is_null());
        assert!(json.as_object().unwrap().contains_key("resume_at"));
    }

    fn overview() -> mestier_core::RunOverview {
        let instant = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("valid instant");
        mestier_core::RunOverview {
            run: Run {
                id: Uuid::from_u128(1),
                org_id: OrganizationId(Uuid::from_u128(2)),
                workflow_id: Uuid::from_u128(3),
                workflow_version_id: Uuid::from_u128(4),
                trigger_event_id: Some(Uuid::from_u128(5)),
                trigger_payload: None,
                trigger_id: Some("t1".to_string()),
                status: RunStatus::Failed,
                error: Some("boom".to_string()),
                next_attempt_at: None,
                locked_at: None,
                locked_by: None,
                started_at: Some(instant),
                finished_at: Some(instant),
                created_at: instant,
            },
            number: 1285,
            steps_done: 3,
            steps_total: 5,
            trigger_event_name: Some("quote.accepted".to_string()),
            failed_connector_id: Some("c4".to_string()),
            workflow_version_created_at: instant,
        }
    }

    #[test]
    fn a_run_response_carries_the_numbering_progress_and_trigger_fields() {
        let json = serde_json::to_value(RunResponse::from(overview())).unwrap();

        assert_eq!(json["number"], 1285);
        assert_eq!(json["steps_done"], 3);
        assert_eq!(json["steps_total"], 5);
        assert_eq!(json["trigger_id"], "t1");
        assert_eq!(json["trigger_event_name"], "quote.accepted");
        assert_eq!(json["failed_connector_id"], "c4");
        assert_eq!(json["workflow_version_created_at"], "2023-11-14T22:13:20Z");
        assert_eq!(json["status"], "failed");
        assert_eq!(json["error"], "boom");
    }

    #[test]
    fn the_optional_run_fields_serialize_as_null_for_a_manual_successful_run() {
        let mut manual = overview();
        manual.run.trigger_event_id = None;
        manual.run.trigger_id = None;
        manual.run.status = RunStatus::Succeeded;
        manual.run.error = None;
        manual.trigger_event_name = None;
        manual.failed_connector_id = None;

        let json = serde_json::to_value(RunResponse::from(manual)).unwrap();

        assert_eq!(json["trigger_id"], serde_json::Value::Null);
        assert_eq!(json["trigger_event_name"], serde_json::Value::Null);
        assert_eq!(json["failed_connector_id"], serde_json::Value::Null);
    }

    #[test]
    fn a_run_detail_response_repeats_the_same_fields_beside_its_steps() {
        let detail = RunDetailResponse {
            run: RunResponse::from(overview()),
            steps: Vec::new(),
            graph: None,
        };

        let json = serde_json::to_value(detail).unwrap();

        for field in [
            "number",
            "steps_done",
            "steps_total",
            "trigger_id",
            "trigger_event_name",
            "failed_connector_id",
            "workflow_version_created_at",
        ] {
            assert!(json.get(field).is_some(), "{field} missing: {json}");
        }
        assert_eq!(json["steps"], json!([]));
    }

    fn webhook_dto() -> TriggerKindDto {
        TriggerKindDto::Webhook {
            method: WebhookMethodDto::Post,
            path: "paiement-recu".to_string(),
            credential_id: Some(Uuid::from_u128(7)),
            contract: vec![ContractKeyDto {
                name: "montant".to_string(),
                kind: ContractKindDto::Number,
                required: true,
                example: "2340".to_string(),
            }],
            on_invalid: OnInvalidDto::Accept,
        }
    }

    fn schedule_dto() -> TriggerKindDto {
        TriggerKindDto::Schedule {
            frequency: FrequencyDto::Weekly,
            time: "09:00".to_string(),
            weekday: Some(WeekdayDto::Monday),
            timezone: Some("Europe/Paris".to_string()),
        }
    }

    #[test]
    fn the_webhook_and_schedule_dtos_have_the_wire_shape_of_the_domain() {
        for dto in [webhook_dto(), schedule_dto()] {
            let core: mestier_core::TriggerKind = dto.clone().into();

            assert_eq!(
                serde_json::to_value(&dto).unwrap(),
                serde_json::to_value(&core).unwrap()
            );
            assert_eq!(TriggerKindDto::from(core), dto);
        }
    }

    #[test]
    fn a_webhook_dto_spells_its_fields_the_way_the_editor_sends_them() {
        assert_eq!(
            serde_json::to_value(webhook_dto()).unwrap(),
            json!({
                "Webhook": {
                    "method": "POST",
                    "path": "paiement-recu",
                    "credential_id": "00000000-0000-0000-0000-000000000007",
                    "contract": [
                        { "name": "montant", "kind": "Number", "required": true, "example": "2340" }
                    ],
                    "on_invalid": "Accept"
                }
            })
        );
        assert_eq!(
            serde_json::to_value(schedule_dto()).unwrap(),
            json!({
                "Schedule": {
                    "frequency": "Weekly",
                    "time": "09:00",
                    "weekday": "Monday",
                    "timezone": "Europe/Paris"
                }
            })
        );
    }

    #[test]
    fn optional_fields_of_the_new_triggers_may_be_left_out() {
        let webhook: TriggerKindDto =
            serde_json::from_value(json!({ "Webhook": { "method": "GET", "path": "ping" } }))
                .expect("a bare webhook reads");
        let schedule: TriggerKindDto = serde_json::from_value(
            json!({ "Schedule": { "frequency": "Daily", "time": "09:00" } }),
        )
        .expect("a bare schedule reads");

        assert_eq!(
            webhook,
            TriggerKindDto::Webhook {
                method: WebhookMethodDto::Get,
                path: "ping".to_string(),
                credential_id: None,
                contract: Vec::new(),
                on_invalid: OnInvalidDto::Reject,
            }
        );
        assert_eq!(
            schedule,
            TriggerKindDto::Schedule {
                frequency: FrequencyDto::Daily,
                time: "09:00".to_string(),
                weekday: None,
                timezone: None,
            }
        );
    }

    #[test]
    fn a_graph_without_the_new_triggers_still_reads_and_writes_as_before() {
        let stored = json!({
            "connectors": [],
            "edges": [],
            "triggers": [
                { "id": "t1", "kind": "Manual" },
                { "id": "t2", "kind": { "Events": ["quote.accepted"] } }
            ]
        });

        let graph: GraphDto = serde_json::from_value(stored).expect("an older graph still reads");

        assert_eq!(graph.triggers[0].kind, TriggerKindDto::Manual);
        assert_eq!(
            graph.triggers[1].kind,
            TriggerKindDto::Events(vec!["quote.accepted".to_string()])
        );
    }

    #[test]
    fn trigger_errors_name_the_trigger_and_the_field_at_fault() {
        use mestier_core::WebhookPathIssue;

        let cases = [
            (
                GraphError::InvalidWebhookPath {
                    trigger_id: "t1".to_string(),
                    issue: WebhookPathIssue::Empty,
                },
                "path",
            ),
            (
                GraphError::DuplicateWebhookPath {
                    trigger_id: "t1".to_string(),
                    path: "x".to_string(),
                },
                "path",
            ),
            (
                GraphError::UnknownWebhookCredential {
                    trigger_id: "t1".to_string(),
                    credential_id: Uuid::from_u128(1),
                },
                "credential",
            ),
            (
                GraphError::WebhookCredentialSchemeNotAccepted {
                    trigger_id: "t1".to_string(),
                    credential_id: Uuid::from_u128(1),
                    scheme: "odoo_api".to_string(),
                },
                "credential",
            ),
            (
                GraphError::DuplicateContractKey {
                    trigger_id: "t1".to_string(),
                    name: "montant".to_string(),
                },
                "contract.montant",
            ),
            (
                GraphError::EmptyContractKeyName {
                    trigger_id: "t1".to_string(),
                },
                "contract",
            ),
            (
                GraphError::InvalidScheduleTime {
                    trigger_id: "t1".to_string(),
                    time: "25:00".to_string(),
                },
                "time",
            ),
            (
                GraphError::InvalidContractKeyName {
                    trigger_id: "t1".to_string(),
                    name: "1 montant".to_string(),
                },
                "contract.1 montant",
            ),
            (
                GraphError::UnknownScheduleTimezone {
                    trigger_id: "t1".to_string(),
                    timezone: "Mars/Olympus".to_string(),
                },
                "timezone",
            ),
        ];

        for (error, field) in cases {
            let response = GraphErrorResponse::from(&error);

            assert_eq!(response.trigger_id.as_deref(), Some("t1"), "{error}");
            assert_eq!(response.connector_id, None, "{error}");
            assert_eq!(response.field.as_deref(), Some(field), "{error}");
            assert!(response.message.contains("t1"), "{}", response.message);
        }
    }

    #[test]
    fn a_graph_holding_too_many_triggers_is_a_graph_level_error() {
        let response = GraphErrorResponse::from(&GraphError::TooManyTriggers);

        assert_eq!(response.connector_id, None);
        assert_eq!(response.trigger_id, None);
        assert_eq!(response.field.as_deref(), Some("triggers"));
        assert!(response.message.contains("20"), "{}", response.message);
    }
}
