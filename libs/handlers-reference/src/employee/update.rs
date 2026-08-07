use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{EmployeeId, LinkEmployeeUserCommand, UpdateEmployeeCommand, UserId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EmployeePath, require_org_membership, response::EmployeeResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEmployeeRequest {
    pub last_name: String,
    /// `null` means "not provided" — see `Employee::first_name`.
    #[serde(default)]
    pub first_name: Option<String>,
    /// `null` means the rate is not set yet; `0` means genuinely free.
    pub hourly_rate_cents: Option<i32>,
    #[serde(default)]
    pub weekly_contract_minutes: i32,
    pub user_id: Option<UserId>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/employees/{employee_id}",
    operation_id = "updateEmployee",
    tag = super::super::TAG,
    params(
        ("employee_id" = EmployeeId, Path, description = "Employee identifier"),
    ),
    request_body = UpdateEmployeeRequest,
    responses(
        (status = 200, description = "Employee updated", body = inline(DataEnvelope<EmployeeResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Employee not found"),
        (status = 409, description = "Employee conflict"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    EmployeePath { employee_id }: EmployeePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateEmployeeRequest>,
) -> Result<Response<EmployeeResponse>, ApiError> {
    let current = state.usecase.get_employee(employee_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;

    state
        .usecase
        .update_employee(UpdateEmployeeCommand {
            id: employee_id,
            last_name: payload.last_name,
            first_name: payload.first_name,
            hourly_rate_cents: payload.hourly_rate_cents,
            weekly_contract_minutes: payload.weekly_contract_minutes,
        })
        .await?;
    let employee = state
        .usecase
        .link_employee_user(LinkEmployeeUserCommand {
            id: employee_id,
            user_id: payload.user_id,
        })
        .await?;

    Ok(Response::OK(employee.into()))
}
