//! Assignment reports, the manager's half of the correction loop.
//!
//! The worker's half — filing, amending, withdrawing — lives in
//! `libs/handlers-field`: two callers, two crates, because they are two
//! different acts by two different people. This crate only ever reads and
//! arbitrates.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState};
use mestier_core::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, MemberId, OrganizationId,
    TaskAssignmentId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::require_org_membership;

pub mod list;
pub mod resolve;

/// This aggregate's routes, unlayered — mirrors `task_comment::router`'s own
/// note: `lib.rs` merges every aggregate submodule before applying the
/// shared rate-limit/auth middleware once.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_patch(resolve::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/assignment-reports")]
pub struct AssignmentReportsPath {
    pub organization_id: OrganizationId,
}

/// Bare id — the manager reaches a report from a list or from the task
/// sheet, never by constructing the organization into the URL themselves.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/assignment-reports/{assignment_report_id}/resolution")]
pub struct AssignmentReportResolutionPath {
    pub assignment_report_id: AssignmentReportId,
}

/// Loads the report and checks the caller belongs to *its* organization —
/// the id is bare, so the organization has to come from the row, never be
/// trusted from the path. Mirrors `require_member_target`.
pub(crate) async fn require_assignment_report(
    state: &AppState,
    identity: &Identity,
    assignment_report_id: AssignmentReportId,
) -> Result<AssignmentReport, ApiError> {
    let report = state
        .usecase
        .get_assignment_report(assignment_report_id)
        .await?;
    require_org_membership(state, identity, report.organization_id).await?;
    Ok(report)
}

/// The caller's own seat in `organization_id`. `require_org_membership`
/// only proves membership; resolving *who* arbitrated is what
/// `resolved_by` needs.
pub(crate) async fn resolve_caller_member(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<MemberId, ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;
    let member = state
        .usecase
        .find_membership(organization_id, user.id)
        .await?
        .ok_or(ApiError::Forbidden)?;
    Ok(member.id)
}

/// The manager's view of a report — never a rate or a cost, same as the
/// field app's own response: applying moves money, but this row does not
/// say how much.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AssignmentReportResponse {
    pub id: AssignmentReportId,
    pub organization_id: OrganizationId,
    pub task_assignment_id: TaskAssignmentId,
    pub reported_minutes: u32,
    pub comment: Option<String>,
    pub reported_by: MemberId,
    pub resolution: AssignmentReportResolution,
    pub resolved_by: Option<MemberId>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AssignmentReport> for AssignmentReportResponse {
    fn from(value: AssignmentReport) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            task_assignment_id: value.task_assignment_id,
            reported_minutes: value.reported_minutes,
            comment: value.comment,
            reported_by: value.reported_by,
            resolution: value.resolution,
            resolved_by: value.resolved_by,
            resolved_at: value.resolved_at,
            resolution_note: value.resolution_note,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
