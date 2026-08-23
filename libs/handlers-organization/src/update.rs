use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{OrganizationId, UpdateOrganizationCommand, application::policy};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrganizationPath, resolve_identity_user, response::OrganizationResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub slug: String,
    /// Whether the field app's home screen offers clocking in/out — see
    /// `Organization::field_clock_enabled`'s own doc. Echoed on every save,
    /// like `name`/`slug`: this is a full replace, not a `PATCH` delta.
    pub field_clock_enabled: bool,
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
    let user = resolve_identity_user(&state, &identity).await?;
    let iam_roles = match &identity {
        Identity::User(u) => u.roles.clone(),
        Identity::Client(c) => c.roles.clone(),
    };
    let actor = policy::user_subject(user.id, iam_roles);

    let organization = state
        .usecase
        .acting_as(user.id)
        .update_organization(UpdateOrganizationCommand {
            actor,
            id: organization_id,
            name: payload.name,
            slug: payload.slug,
            field_clock_enabled: payload.field_clock_enabled,
        })
        .await?;

    Ok(Response::OK(organization.into()))
}
