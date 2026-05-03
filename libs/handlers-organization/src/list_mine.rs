use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::CurrentUserOrganizationsPath, response::OrganizationResponse};

#[utoipa::path(
    get,
    path = "/api/v1/users/@me/organizations",
    operation_id = "listMyOrganizations",
    tag = super::TAG,
    responses(
        (status = 200, description = "Organizations the authenticated user belongs to", body = inline(DataEnvelope<Vec<OrganizationResponse>>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    _: CurrentUserOrganizationsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<OrganizationResponse>>, ApiError> {
    let sub = identity.id();

    let organizations = state
        .usecase
        .list_organizations_for_user(sub)
        .await?
        .into_iter()
        .map(OrganizationResponse::from)
        .collect();

    Ok(Response::OK(organizations))
}
