use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateLegalMentionTemplateCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
	paths::LegalMentionTemplatesPath,
	require_org_membership,
	response::LegalMentionTemplateResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLegalMentionTemplateRequest {
	pub name: String,
	pub body: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/legal-mention-templates",
    operation_id = "createLegalMentionTemplate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateLegalMentionTemplateRequest,
    responses(
        (status = 201, description = "Template created", body = inline(DataEnvelope<LegalMentionTemplateResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
	path: LegalMentionTemplatesPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<CreateLegalMentionTemplateRequest>,
) -> Result<Response<LegalMentionTemplateResponse>, ApiError> {
	require_org_membership(&state, &identity, path.organization_id).await?;

	let template = state
		.usecase
		.create_legal_mention_template(CreateLegalMentionTemplateCommand {
			org_id: path.organization_id,
			name: payload.name,
			body: payload.body,
		})
		.await?;

	Ok(Response::Created(template.into()))
}
