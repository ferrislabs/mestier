//! Employee absences: leave, sick leave and unavailability. Same business
//! fact, qualified by `kind` — one table, one HTTP resource. No `get_one`:
//! the API only ever hands absences back as part of a list or a mutation
//! response (see the planning module design doc).
//!
//! Moved here from `handlers-planning` — an absence is a fact about an
//! employee, not about how the planning grid is edited; the aggregate itself
//! never left `libs/core` (see the planning remodel design doc's "Absences
//! vers le module RH" decision).
//!
//! Unlike this crate's other aggregates, this module keeps its own
//! `router(state)`, its own `AbsencesPath`/`AbsencePath`, and its own
//! `AbsenceResponse` instead of folding into the shared `paths.rs`/
//! `response.rs`/flat `router()` in `lib.rs` — this workstream's ownership
//! boundary is `absence/**` only, so it stays self-contained rather than
//! editing files it doesn't own. `lib.rs` only gains a `pub mod absence;`
//! and a `.merge(absence::router(state))`.

use auth::Identity;
use axum::{
    Json, Router,
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::{ApiError, AppState};
use http::StatusCode;
use mestier_core::{
    Absence, AbsenceError, AbsenceId, AbsenceKind, AbsenceStatus, AbsenceTypeId, DayPart, MemberId,
    OrganizationId,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::require_org_membership;

pub mod approve;
pub mod cancel;
pub mod create;
pub mod list;
pub mod reject;
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
        .typed_post(approve::handler)
        .typed_post(reject::handler)
        .typed_post(cancel::handler)
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
    pub absence_id: AbsenceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absences/{absence_id}/approve")]
pub struct ApproveAbsencePath {
    pub organization_id: OrganizationId,
    pub absence_id: AbsenceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absences/{absence_id}/reject")]
pub struct RejectAbsencePath {
    pub organization_id: OrganizationId,
    pub absence_id: AbsenceId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absences/{absence_id}/cancel")]
pub struct CancelAbsencePath {
    pub organization_id: OrganizationId,
    pub absence_id: AbsenceId,
}

#[derive(Debug)]
pub enum AbsenceFailure {
    Api(ApiError),
    Refused(AbsenceError),
}

impl From<ApiError> for AbsenceFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<AbsenceError> for AbsenceFailure {
    fn from(error: AbsenceError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for AbsenceFailure {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (
                    status,
                    Json(json!({
                        "code": "E_UNPROCESSABLE_ENTITY",
                        "message": error.to_string(),
                        "status": status.as_u16(),
                        "details": { "field": error.field() },
                    })),
                )
                    .into_response()
            }
        }
    }
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
    absence_id: AbsenceId,
) -> Result<Absence, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let absence = state.usecase.get_absence(absence_id).await?;
    if absence.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(absence)
}

/// Validates that `member_id` actually belongs to `organization_id`
/// before an absence is created for it. Mirrors `handlers-planning`'s own
/// `work_order::require_work_order_targets` and `handlers-quote`'s
/// `require_quote_targets`.
pub(crate) async fn require_member_target(
    state: &AppState,
    organization_id: OrganizationId,
    member_id: MemberId,
) -> Result<(), ApiError> {
    let member = state
        .usecase
        .find_member_seat(member_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if member.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AbsenceResponse {
    pub id: AbsenceId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub kind: AbsenceKind,
    pub type_id: AbsenceTypeId,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub start_date: NaiveDate,
    pub start_part: DayPart,
    pub end_date: NaiveDate,
    pub end_part: DayPart,
    pub status: AbsenceStatus,
    pub note: Option<String>,
    pub proof_file_key: Option<String>,
    pub created_by_member_id: Option<MemberId>,
    pub decided_by_member_id: Option<MemberId>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Absence> for AbsenceResponse {
    fn from(value: Absence) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            kind: value.kind,
            type_id: value.type_id,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            start_date: value.start_date,
            start_part: value.start_part,
            end_date: value.end_date,
            end_part: value.end_part,
            status: value.status,
            note: value.note,
            proof_file_key: value.proof_file_key,
            created_by_member_id: value.created_by_member_id,
            decided_by_member_id: value.decided_by_member_id,
            decided_at: value.decided_at,
            decision_comment: value.decision_comment,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `uuid` is not a direct dependency of `handlers-reference` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn absence() -> Absence {
        let now = Utc::now();
        let id: AbsenceId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let organization_id: OrganizationId =
            "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let member_id: MemberId = "33333333-3333-3333-3333-333333333333".parse().unwrap();
        Absence {
            id,
            organization_id,
            member_id,
            kind: AbsenceKind::Leave,
            type_id: "44444444-4444-4444-4444-444444444444".parse().unwrap(),
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            start_date: now.date_naive(),
            start_part: DayPart::Morning,
            end_date: now.date_naive(),
            end_part: DayPart::Afternoon,
            status: AbsenceStatus::Approved,
            note: Some("Congés payés".to_owned()),
            proof_file_key: None,
            created_by_member_id: None,
            decided_by_member_id: None,
            decided_at: None,
            decision_comment: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn absence_response_carries_the_employee_and_kind() {
        let source = absence();
        let expected_member_id = source.member_id;

        let response: AbsenceResponse = source.into();

        assert_eq!(response.member_id, expected_member_id);
        assert_eq!(response.kind, AbsenceKind::Leave);
    }
}
