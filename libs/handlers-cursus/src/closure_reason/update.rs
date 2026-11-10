use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::UpdateClosureReasonCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ClosureReasonPath, require_org_membership, response::ClosureReasonResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateClosureReasonRequest {
    pub label: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/cursus/closure-reasons/{closure_reason_id}",
    operation_id = "updateCursusClosureReason",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("closure_reason_id" = mestier_core::ClosureReasonId, Path, description = "Closure reason identifier"),
    ),
    request_body = UpdateClosureReasonRequest,
    responses(
        (status = 200, description = "The closure reason was updated", body = inline(DataEnvelope<ClosureReasonResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Closure reason not found in the organization"),
        (status = 409, description = "Invalid closure reason"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ClosureReasonPath {
        organization_id,
        closure_reason_id,
    }: ClosureReasonPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateClosureReasonRequest>,
) -> Result<Response<ClosureReasonResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let updated = state
        .usecase
        .acting_as(user_id)
        .update_closure_reason(UpdateClosureReasonCommand {
            actor,
            organization_id,
            id: closure_reason_id,
            label: payload.label,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
