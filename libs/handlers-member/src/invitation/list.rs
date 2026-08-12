use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::application::policy;

use crate::{paths::InvitationsPath, response::InvitationResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/invitations",
    operation_id = "listInvitations",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Pending invitations for the organization", body = inline(DataEnvelope<Vec<InvitationResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0), err)]
pub async fn handler(
    path: InvitationsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<InvitationResponse>>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    let invitations = state
        .usecase
        .list_pending_invitations(actor, path.organization_id)
        .await?;

    let items = invitations
        .into_iter()
        .map(InvitationResponse::from)
        .collect();

    Ok(Response::OK(items))
}
