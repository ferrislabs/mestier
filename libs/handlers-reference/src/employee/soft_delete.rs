use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::EmployeeId;

use crate::{EmptyResponse, paths::EmployeePath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/employees/{employee_id}",
    operation_id = "deleteEmployee",
    tag = super::super::TAG,
    params(
        ("employee_id" = EmployeeId, Path, description = "Employee identifier"),
    ),
    responses(
        (status = 204, description = "Employee soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Employee not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EmployeePath { employee_id }: EmployeePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let current = state.usecase.get_employee(employee_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    state.usecase.soft_delete_employee(employee_id).await?;

    Ok(Response::NoContent)
}
