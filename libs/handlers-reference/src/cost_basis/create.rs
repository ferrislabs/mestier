use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::cost_basis::{EmployeeCostBasesPath, EmployeeCostBasisResponse};

/// Dates a cost basis change: closes the open version at `effective_from`
/// and opens a new one, or — when `effective_from` matches the open
/// version's own — edits it in place, so calling this twice for the same
/// date never accumulates a second row.
///
/// The existing `PUT /members/{member_id}/employee-profile` route keeps
/// working and stays undated: it means "from today", which is what a caller
/// that does not state a date means. This route is for a caller who does.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetEmployeeCostBasisRequest {
    pub effective_from: NaiveDate,
    /// `null` means the rate is not set yet; `0` means genuinely free.
    /// Ignored when `is_salaried` is set.
    #[serde(default)]
    pub hourly_rate_cents: Option<i32>,
    /// Required, deliberately, the same reason `UpsertEmployeeProfileRequest`
    /// requires it: a caller that does not state the cost basis has not
    /// thought about it, and a 400 says so.
    pub is_salaried: bool,
    #[serde(default)]
    pub monthly_cost_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/employees/{employee_id}/cost-bases",
    operation_id = "setEmployeeCostBasis",
    tag = super::super::TAG,
    params(
        ("employee_id" = mestier_core::EmployeeId, Path, description = "Employee identifier"),
    ),
    request_body = SetEmployeeCostBasisRequest,
    responses(
        (status = 201, description = "Cost basis version dated", body = inline(DataEnvelope<EmployeeCostBasisResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Employee not found"),
        (status = 409, description = "A date before the currently open version, or an overlapping range"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EmployeeCostBasesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetEmployeeCostBasisRequest>,
) -> Result<Response<EmployeeCostBasisResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let basis = state
        .usecase
        .acting_as(user_id)
        .set_employee_cost_basis(
            actor,
            path.employee_id,
            payload.effective_from,
            payload.is_salaried,
            payload.hourly_rate_cents,
            payload.monthly_cost_cents,
            payload.weekly_contract_minutes,
        )
        .await?;

    Ok(Response::Created(basis.into()))
}
