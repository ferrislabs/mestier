use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, IdentityExt, Response};
use mestier_core::{OrganizationId, UpdateOrganizationCommand, application::policy};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrganizationPath, response::OrganizationResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub slug: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}",
    operation_id = "updateOrganization",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated", body = inline(DataEnvelope<OrganizationResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
        (status = 409, description = "Slug already taken"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    OrganizationPath { organization_id }: OrganizationPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    let user_id = identity.user_id()?;
    let iam_roles = match &identity {
        Identity::User(u) => u.roles.clone(),
        Identity::Client(c) => c.roles.clone(),
    };
    let actor = policy::user_subject(user_id, iam_roles);

    let organization = state
        .usecase
        .update_organization(UpdateOrganizationCommand {
            actor,
            id: organization_id,
            name: payload.name,
            slug: payload.slug,
        })
        .await?;

    Ok(Response::OK(organization.into()))
}
