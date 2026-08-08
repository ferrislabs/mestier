use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{ReplaceRhythmCommand, RhythmSlotInput};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    work_time::{RhythmPath, RhythmResponse},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct RhythmSlotRequest {
    pub weekday: i16,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// The full weekly rhythm to persist: replaces whatever version is
/// currently in effect, or opens a new one if `effective_from` moves past
/// it — see `WorkTimeService::replace_rhythm` for the exact rules.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PutRhythmRequest {
    pub effective_from: NaiveDate,
    #[serde(default)]
    pub effective_to: Option<NaiveDate>,
    pub slots: Vec<RhythmSlotRequest>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/employees/{employee_id}/rhythm",
    operation_id = "putRhythm",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("employee_id" = mestier_core::EmployeeId, Path, description = "Employee identifier"),
    ),
    request_body = PutRhythmRequest,
    responses(
        (status = 200, description = "Rhythm replaced", body = inline(DataEnvelope<RhythmResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Rhythm conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RhythmPath {
        organization_id,
        employee_id,
    }: RhythmPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PutRhythmRequest>,
) -> Result<Response<RhythmResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;
    crate::require_employee_target(&state, organization_id, employee_id).await?;

    let rhythm = state
        .usecase
        .acting_as(actor)
        .replace_rhythm(ReplaceRhythmCommand {
            organization_id,
            employee_id,
            effective_from: payload.effective_from,
            effective_to: payload.effective_to,
            slots: payload
                .slots
                .into_iter()
                .map(|slot| RhythmSlotInput {
                    weekday: slot.weekday,
                    starts_minute: slot.starts_minute,
                    ends_minute: slot.ends_minute,
                })
                .collect(),
        })
        .await?;

    Ok(Response::OK(rhythm.into()))
}
