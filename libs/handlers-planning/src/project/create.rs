use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateProjectCommand, CustomerContextId, CustomerId, QuoteId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    project::{ProjectsPath, require_project_targets},
    require_org_membership,
    response::ProjectResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    /// Absent for internal work. Leaving it out is a statement, not an omission
    /// somebody forgot to fill in.
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    #[serde(default)]
    pub customer_context_id: Option<CustomerContextId>,
    #[serde(default)]
    pub quote_id: Option<QuoteId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/projects",
    operation_id = "createProject",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created", body = inline(DataEnvelope<ProjectResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A blank name, or a context with no customer"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ProjectsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Response<ProjectResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    require_project_targets(
        &state,
        path.organization_id,
        payload.customer_id,
        payload.customer_context_id,
        payload.quote_id,
    )
    .await?;

    let project = state
        .usecase
        .create_project(CreateProjectCommand {
            organization_id: path.organization_id,
            name: payload.name,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            quote_id: payload.quote_id,
        })
        .await?;

    Ok(Response::Created(project.into()))
}
