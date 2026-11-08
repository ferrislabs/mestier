use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::DecideAbsenceCommand;

use crate::absence::{AbsenceFailure, AbsenceResponse, CancelAbsencePath};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/absences/{absence_id}/cancel",
    operation_id = "cancelAbsence",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("absence_id" = mestier_core::AbsenceId, Path, description = "Absence identifier"),
    ),
    responses(
        (status = 200, description = "Absence cancelled", body = inline(DataEnvelope<AbsenceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Neither the creator nor a planning manager"),
        (status = 404, description = "Absence not found in the organization"),
        (status = 422, description = "Already rejected or cancelled"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CancelAbsencePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AbsenceResponse>, AbsenceFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let absence = state
        .usecase
        .acting_as(user_id)
        .cancel_absence(DecideAbsenceCommand {
            actor,
            organization_id: path.organization_id,
            id: path.absence_id,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(absence.into()))
}
