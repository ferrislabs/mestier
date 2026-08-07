use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateEmployeeCommand, UserId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EmployeesPath, require_org_membership, response::EmployeeResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEmployeeRequest {
    pub user_id: Option<UserId>,
    pub last_name: String,
    /// `null` means "not provided" — see `Employee::first_name`.
    #[serde(default)]
    pub first_name: Option<String>,
    /// `null` means the rate is not set yet; `0` means genuinely free.
    pub hourly_rate_cents: Option<i32>,
    #[serde(default)]
    pub weekly_contract_minutes: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/employees",
    operation_id = "createEmployee",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateEmployeeRequest,
    responses(
        (status = 201, description = "Employee created", body = inline(DataEnvelope<EmployeeResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Employee conflict"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0), err)]
pub async fn handler(
    path: EmployeesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateEmployeeRequest>,
) -> Result<Response<EmployeeResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let employee = state
        .usecase
        .create_employee(CreateEmployeeCommand {
            organization_id: path.organization_id,
            user_id: payload.user_id,
            last_name: payload.last_name,
            first_name: payload.first_name,
            hourly_rate_cents: payload.hourly_rate_cents,
            weekly_contract_minutes: payload.weekly_contract_minutes,
        })
        .await?;

    Ok(Response::Created(employee.into()))
}
