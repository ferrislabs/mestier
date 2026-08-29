use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EmployeeProfilePath, response::EmployeeResponse};

/// A contract, and nothing else. No name, no account: both belong to the seat,
/// and there is deliberately no way to set them from here.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertEmployeeProfileRequest {
    /// `null` means the rate is not set yet; `0` means genuinely free. The two
    /// are never conflated — a cost computation refuses to produce a figure for
    /// the first and happily produces zero for the second.
    #[serde(default)]
    pub hourly_rate_cents: Option<i32>,
    /// True when this person is costed from `monthly_cost_cents` rather than by
    /// the hour. Never means their time is free: an `hourly_rate_cents` given
    /// alongside this is not stored, and neither is a `monthly_cost_cents` given
    /// without it — see `EmployeeService::upsert_employee_profile`.
    ///
    /// Required, deliberately, unlike the other optional fields. This is a `PUT`:
    /// omitting the flag used to default it to `false`, which silently moved a
    /// salaried person back onto an hourly basis and cleared the amount somebody
    /// had just entered. A caller that does not state the cost basis is a caller
    /// that has not thought about it, and a 400 says so.
    pub is_salaried: bool,
    /// Monthly employer cost, loaded with contributions, for a salaried person.
    /// The hourly equivalent is derived from this and `weekly_contract_minutes`,
    /// never stored.
    #[serde(default)]
    pub monthly_cost_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

/// `PUT`, not `POST`: attaching a contract to a member who already has one
/// replaces it rather than opening a second, and calling this twice with the
/// same body leaves the same single profile behind.
#[utoipa::path(
    put,
    path = "/api/v1/members/{member_id}/employee-profile",
    operation_id = "upsertEmployeeProfile",
    tag = super::super::TAG,
    params(
        ("member_id" = mestier_core::MemberId, Path, description = "Member identifier"),
    ),
    request_body = UpsertEmployeeProfileRequest,
    responses(
        (status = 200, description = "Employee profile attached", body = inline(DataEnvelope<EmployeeResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(member_id = %path.member_id.0), err)]
pub async fn handler(
    path: EmployeeProfilePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpsertEmployeeProfileRequest>,
) -> Result<Response<EmployeeResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    // The seat is loaded and authorized against inside the use case, never
    // against an organization taken from the path — see
    // `MestierUseCase::upsert_employee_profile`.
    let employee = state
        .usecase
        .acting_as(user_id)
        .upsert_employee_profile(
            actor,
            path.member_id,
            payload.hourly_rate_cents,
            payload.is_salaried,
            payload.monthly_cost_cents,
            payload.weekly_contract_minutes,
        )
        .await?;

    Ok(Response::OK(employee.into()))
}
