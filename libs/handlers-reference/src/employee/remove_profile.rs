use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};

use crate::{EmptyResponse, paths::EmployeeProfilePath};

/// Detaches the contract from a member. The seat survives, and so does
/// everything hanging off it — assignments, absences, work slots. This removes
/// a contract, never a person; removing the person is `DELETE /members/{id}`.
#[utoipa::path(
    delete,
    path = "/api/v1/members/{member_id}/employee-profile",
    operation_id = "removeEmployeeProfile",
    tag = super::super::TAG,
    params(
        ("member_id" = mestier_core::MemberId, Path, description = "Member identifier"),
    ),
    responses(
        (status = 204, description = "Employee profile detached"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member has no employee profile"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(member_id = %member_id.0), err)]
pub async fn handler(
    EmployeeProfilePath { member_id }: EmployeeProfilePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .remove_employee_profile(actor, member_id)
        .await?;

    Ok(Response::NoContent)
}
