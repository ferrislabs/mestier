use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::application::policy;

use crate::cost_basis::{EmployeeCostBasesPath, EmployeeCostBasisResponse};

/// Not paginated: an employee's history is a handful of rows, not a
/// collection that grows with the organization. Same gate as `member.manage`
/// everywhere else this figure is read — see
/// `MestierUseCase::list_employee_cost_bases`.
#[utoipa::path(
    get,
    path = "/api/v1/employees/{employee_id}/cost-bases",
    operation_id = "listEmployeeCostBases",
    tag = super::super::TAG,
    params(
        ("employee_id" = mestier_core::EmployeeId, Path, description = "Employee identifier"),
    ),
    responses(
        (status = 200, description = "The employee's cost history, oldest first", body = inline(DataEnvelope<Vec<EmployeeCostBasisResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Employee not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EmployeeCostBasesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<EmployeeCostBasisResponse>>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    let history = state
        .usecase
        .acting_as(user_id)
        .list_employee_cost_bases(actor, path.employee_id)
        .await?;
    let items: Vec<EmployeeCostBasisResponse> = history
        .into_iter()
        .map(EmployeeCostBasisResponse::from)
        .collect();

    Ok(Response::OK(items))
}
