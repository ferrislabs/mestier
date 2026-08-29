use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CustomerContextId, CustomerId, QuoteId, UpdateProjectCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    project::{ProjectPath, require_project, require_project_targets},
    response::ProjectResponse,
};

/// Every field is the new complete value, and every one of them is required.
///
/// A partial `PATCH` would need the absent/null distinction for three nullable
/// links, and a project has few enough fields that sending them all is honest:
/// clearing the customer means sending `null`, not omitting the key and hoping.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/projects/{project_id}",
    operation_id = "patchProject",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_id" = mestier_core::ProjectId, Path, description = "Project identifier"),
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated", body = inline(DataEnvelope<ProjectResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "A blank name, or a context with no customer"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectPath {
        organization_id,
        project_id,
    }: ProjectPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Response<ProjectResponse>, ApiError> {
    require_project(&state, &identity, organization_id, project_id).await?;
    require_project_targets(
        &state,
        organization_id,
        payload.customer_id,
        payload.customer_context_id,
        payload.quote_id,
    )
    .await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let project = state
        .usecase
        .acting_as(user_id)
        .update_project(UpdateProjectCommand {
            actor,
            id: project_id,
            name: payload.name,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            quote_id: payload.quote_id,
        })
        .await?;

    Ok(Response::OK(project.into()))
}
