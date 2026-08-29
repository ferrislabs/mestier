use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};

use crate::absence::{AbsencePath, require_absence};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/absences/{absence_id}",
    operation_id = "deleteAbsence",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("absence_id" = mestier_core::AbsenceId, Path, description = "Absence identifier"),
    ),
    responses(
        (status = 204, description = "Absence soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Absence not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AbsencePath {
        organization_id,
        absence_id,
    }: AbsencePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_absence(&state, &identity, organization_id, absence_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .soft_delete_absence(actor, absence_id)
        .await?;

    Ok(Response::NoContent)
}
