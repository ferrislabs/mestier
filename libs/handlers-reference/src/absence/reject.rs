use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::RejectAbsenceCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::absence::{AbsenceFailure, AbsenceResponse, RejectAbsencePath};

#[derive(Debug, Deserialize, ToSchema)]
pub struct RejectAbsenceRequest {
    pub comment: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/absences/{absence_id}/reject",
    operation_id = "rejectAbsence",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("absence_id" = mestier_core::AbsenceId, Path, description = "Absence identifier"),
    ),
    request_body = RejectAbsenceRequest,
    responses(
        (status = 200, description = "Absence rejected", body = inline(DataEnvelope<AbsenceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Absence not found in the organization"),
        (status = 422, description = "Blank comment, or already decided"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: RejectAbsencePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RejectAbsenceRequest>,
) -> Result<Response<AbsenceResponse>, AbsenceFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let absence = state
        .usecase
        .acting_as(user_id)
        .reject_absence(RejectAbsenceCommand {
            actor,
            organization_id: path.organization_id,
            id: path.absence_id,
            comment: payload.comment,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(absence.into()))
}
