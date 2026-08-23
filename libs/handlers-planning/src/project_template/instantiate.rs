use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CustomerContextId, CustomerId, InstantiateProjectTemplateCommand, QuoteId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    project::require_project_targets,
    project_template::{ProjectTemplateInstantiatePath, require_project_template},
    response::InstantiateProjectTemplateResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct InstantiateProjectTemplateRequest {
    pub name: String,
    pub start_date: NaiveDate,
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    #[serde(default)]
    pub customer_context_id: Option<CustomerContextId>,
    #[serde(default)]
    pub quote_id: Option<QuoteId>,
}

/// Turns a template into a real project with real tasks in one transaction.
/// `start_date` is the only date the caller gives: every task shape's
/// `day_offset` resolves against it, in the organization's own timezone.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/instantiate",
    operation_id = "instantiateProjectTemplate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("project_template_id" = mestier_core::ProjectTemplateId, Path, description = "Project template identifier"),
    ),
    request_body = InstantiateProjectTemplateRequest,
    responses(
        (status = 201, description = "Project and tasks created from the template", body = inline(DataEnvelope<InstantiateProjectTemplateResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
        (status = 409, description = "A blank name, or an archived template"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectTemplateInstantiatePath {
        organization_id,
        project_template_id,
    }: ProjectTemplateInstantiatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<InstantiateProjectTemplateRequest>,
) -> Result<Response<InstantiateProjectTemplateResponse>, ApiError> {
    require_project_template(&state, &identity, organization_id, project_template_id).await?;
    require_project_targets(
        &state,
        organization_id,
        payload.customer_id,
        payload.customer_context_id,
        payload.quote_id,
    )
    .await?;

    let (project, tasks) = state
        .usecase
        .instantiate_project_template(InstantiateProjectTemplateCommand {
            template_id: project_template_id,
            organization_id,
            name: payload.name,
            start_date: payload.start_date,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            quote_id: payload.quote_id,
        })
        .await?;

    Ok(Response::Created(InstantiateProjectTemplateResponse {
        project: project.into(),
        tasks: tasks.into_iter().map(Into::into).collect(),
    }))
}
