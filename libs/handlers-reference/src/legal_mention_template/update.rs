use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{LegalMentionTemplateId, UpdateLegalMentionTemplateCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
	paths::LegalMentionTemplatePath,
	require_org_membership,
	response::LegalMentionTemplateResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLegalMentionTemplateRequest {
	pub name: String,
	pub body: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/legal-mention-templates/{template_id}",
    operation_id = "updateLegalMentionTemplate",
    tag = super::super::TAG,
    params(
        ("template_id" = LegalMentionTemplateId, Path, description = "Template identifier"),
    ),
    request_body = UpdateLegalMentionTemplateRequest,
    responses(
        (status = 200, description = "Template updated", body = inline(DataEnvelope<LegalMentionTemplateResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
	LegalMentionTemplatePath { template_id }: LegalMentionTemplatePath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<UpdateLegalMentionTemplateRequest>,
) -> Result<Response<LegalMentionTemplateResponse>, ApiError> {
	let current = state
		.usecase
		.get_legal_mention_template(template_id)
		.await?;
	require_org_membership(&state, &identity, current.org_id).await?;

	let template = state
		.usecase
		.update_legal_mention_template(UpdateLegalMentionTemplateCommand {
			id: template_id,
			name: payload.name,
			body: payload.body,
		})
		.await?;

	Ok(Response::OK(template.into()))
}
