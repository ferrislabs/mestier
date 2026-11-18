use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateDafCommand, DafDesignation};

use crate::{
    daf::{ProjectDafsPath, request::CreateDafRequest, response::DafResponse},
    require_org_membership,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}/dafs",
    operation_id = "createDaf",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    request_body = CreateDafRequest,
    responses(
        (status = 201, description = "The next DAF of the project", body = inline(DataEnvelope<DafResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project or product not found"),
        (status = 409, description = "Neither a designation nor a product, a designation over 200 characters, or the limit of 200 DAF per project"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectDafsPath {
        organization_id,
        project_id,
    }: ProjectDafsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateDafRequest>,
) -> Result<Response<DafResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let designation = payload
        .designation
        .filter(|text| !text.trim().is_empty())
        .map(DafDesignation::try_from)
        .transpose()?;
    let daf = state
        .usecase
        .create_daf(CreateDafCommand {
            actor,
            organization_id,
            project_id,
            designation,
            product_id: payload.product_id,
        })
        .await?;

    Ok(Response::Created(daf.into()))
}
