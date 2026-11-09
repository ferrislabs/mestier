use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::CreateClosureReasonCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::ClosureReasonCollectionPath, require_org_membership, response::ClosureReasonResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateClosureReasonRequest {
    pub label: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/closure-reasons",
    operation_id = "createCursusClosureReason",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateClosureReasonRequest,
    responses(
        (status = 201, description = "The closure reason was created", body = inline(DataEnvelope<ClosureReasonResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Referenced resource not found in the organization"),
        (status = 409, description = "Invalid closure reason"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ClosureReasonCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateClosureReasonRequest>,
) -> Result<Response<ClosureReasonResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let created = state
        .usecase
        .acting_as(user_id)
        .create_closure_reason(CreateClosureReasonCommand {
            actor,
            organization_id: path.organization_id,
            label: payload.label,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
