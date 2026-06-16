use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::EmployeeId;

use crate::{paths::EmployeePath, require_org_membership, response::EmployeeResponse};

#[utoipa::path(
    get,
    path = "/api/v1/employees/{employee_id}",
    operation_id = "getEmployee",
    tag = super::super::TAG,
    params(
        ("employee_id" = EmployeeId, Path, description = "Employee identifier"),
    ),
    responses(
        (status = 200, description = "Employee details", body = inline(DataEnvelope<EmployeeResponse>)),
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
) -> Result<Response<EmployeeResponse>, ApiError> {
    let employee = state.usecase.get_employee(employee_id).await?;
    require_org_membership(&state, &identity, employee.organization_id).await?;

    Ok(Response::OK(employee.into()))
}
