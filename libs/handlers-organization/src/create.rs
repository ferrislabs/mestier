use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateOrganizationCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrganizationsPath, response::OrganizationResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    operation_id = "createOrganization",
    tag = super::TAG,
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created", body = inline(DataEnvelope<OrganizationResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Slug already taken"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(slug = %payload.slug), err)]
pub async fn handler(
    _: OrganizationsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    let owner = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;
    let command = CreateOrganizationCommand {
        name: payload.name,
        slug: payload.slug,
        owner_id: owner.id,
    };

    let org = state.usecase.create_organization(command).await?;

    Ok(Response::Created(org.into()))
}
