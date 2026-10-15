//! The manager's half of the visit loop: enter a visit directly, validate or
//! reject what the field proposed, and list what waits. The field's half is
//! `handlers-field`. See `docs/chantiers/entretien.md`.

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::AppState;
use mestier_core::{
    Intervention, InterventionId, InterventionStatus, MaintenanceLineId, MemberId, OrganizationId,
    TaskId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod list;
pub mod record;
pub mod reject;
pub mod validate;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_post(record::handler)
        .typed_get(list::handler)
        .typed_post(validate::handler)
        .typed_post(reject::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/lines/{line_id}/interventions")]
pub struct LineInterventionsPath {
    pub organization_id: OrganizationId,
    pub line_id: MaintenanceLineId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/maintenance/interventions")]
pub struct InterventionsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/validate"
)]
pub struct ValidateInterventionPath {
    pub organization_id: OrganizationId,
    pub intervention_id: InterventionId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/reject"
)]
pub struct RejectInterventionPath {
    pub organization_id: OrganizationId,
    pub intervention_id: InterventionId,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct InterventionResponse {
    pub id: InterventionId,
    pub line_id: MaintenanceLineId,
    pub done_on: NaiveDate,
    pub status: InterventionStatus,
    /// The task the field declared it on; `None` for a manager's entry.
    pub task_id: Option<TaskId>,
    pub proposed_by: MemberId,
    pub resolved_by: Option<MemberId>,
    pub resolved_at: Option<DateTime<Utc>>,
    /// Why the manager rejected it.
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Intervention> for InterventionResponse {
    fn from(value: Intervention) -> Self {
        Self {
            id: value.id,
            line_id: value.line_id,
            done_on: value.done_on,
            status: value.status,
            task_id: value.task_id,
            proposed_by: value.proposed_by,
            resolved_by: value.resolved_by,
            resolved_at: value.resolved_at,
            note: value.note,
            created_at: value.created_at,
        }
    }
}
