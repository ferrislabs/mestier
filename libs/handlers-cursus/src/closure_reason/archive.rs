use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{
    paths::ClosureReasonArchivePath, require_org_membership, response::ClosureReasonResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/cursus/closure-reasons/{closure_reason_id}/archive",
    operation_id = "archiveCursusClosureReason",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("closure_reason_id" = mestier_core::ClosureReasonId, Path, description = "Closure reason identifier"),
    ),
    responses(
        (status = 200, description = "The closure reason is archived", body = inline(DataEnvelope<ClosureReasonResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Closure reason not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ClosureReasonArchivePath {
        organization_id,
        closure_reason_id,
    }: ClosureReasonArchivePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClosureReasonResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let archived = state
        .usecase
        .acting_as(user_id)
        .archive_closure_reason(actor, organization_id, closure_reason_id)
        .await?;

    Ok(Response::OK(archived.into()))
}
