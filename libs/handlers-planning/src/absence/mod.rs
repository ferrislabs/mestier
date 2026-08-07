//! Employee absences: leave, sick leave and unavailability. Same business
//! fact, qualified by `kind` — one table, one HTTP resource. No `get_one`:
//! the API only ever hands absences back as part of a list or a mutation
//! response (see the planning module design doc).
//!
//! Owns its own leaf paths (see the crate-level `paths.rs` docstring) since
//! `organization_id` and `absence_id` are both part of the single-item
//! route.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState};
use mestier_core::{AbsenceKind, EmployeeAbsence, EmployeeAbsenceId, EmployeeId, OrganizationId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::require_org_membership;

pub mod create;
pub mod list;
pub mod soft_delete;
pub mod update;

/// This aggregate's routes, unlayered — the crate-level `router` in
/// `lib.rs` merges every aggregate submodule's router before applying the
/// shared rate-limit/auth middleware once, rather than each submodule
/// layering its own.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
        .typed_delete(soft_delete::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absences")]
pub struct AbsencesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absences/{absence_id}")]
pub struct AbsencePath {
    pub organization_id: OrganizationId,
    pub absence_id: EmployeeAbsenceId,
}

/// Loads the absence and checks both that the caller belongs to
/// `organization_id` and that the absence actually belongs to it —
/// `organization_id` is part of every route, so a mismatch (a real
/// `absence_id` from a different organization) is treated the same as
/// "does not exist" rather than leaking cross-tenant existence via 403.
pub(crate) async fn require_absence(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    absence_id: EmployeeAbsenceId,
) -> Result<EmployeeAbsence, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let absence = state.usecase.get_absence(absence_id).await?;
    if absence.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(absence)
}

/// Validates that `employee_id` actually belongs to `organization_id`
/// before an absence is created for it. Mirrors `handlers-planning`'s own
/// `work_order::require_work_order_targets` and `handlers-quote`'s
/// `require_quote_targets`.
pub(crate) async fn require_employee_target(
    state: &AppState,
    organization_id: OrganizationId,
    employee_id: EmployeeId,
) -> Result<(), ApiError> {
    let employee = state.usecase.get_employee(employee_id).await?;
    if employee.organization_id != organization_id {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AbsenceResponse {
    pub id: EmployeeAbsenceId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub kind: AbsenceKind,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmployeeAbsence> for AbsenceResponse {
    fn from(value: EmployeeAbsence) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            employee_id: value.employee_id,
            kind: value.kind,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            note: value.note,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `uuid` is not a direct dependency of `handlers-planning` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn absence() -> EmployeeAbsence {
        let now = Utc::now();
        let id: EmployeeAbsenceId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let organization_id: OrganizationId =
            "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let employee_id: EmployeeId = "33333333-3333-3333-3333-333333333333".parse().unwrap();
        EmployeeAbsence {
            id,
            organization_id,
            employee_id,
            kind: AbsenceKind::Leave,
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            note: Some("Congés payés".to_owned()),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn absence_response_carries_the_employee_and_kind() {
        let source = absence();
        let expected_employee_id = source.employee_id;

        let response: AbsenceResponse = source.into();

        assert_eq!(response.employee_id, expected_employee_id);
        assert_eq!(response.kind, AbsenceKind::Leave);
    }
}
