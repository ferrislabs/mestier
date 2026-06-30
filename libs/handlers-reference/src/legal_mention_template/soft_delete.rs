use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::LegalMentionTemplateId;

use crate::{EmptyResponse, paths::LegalMentionTemplatePath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/legal-mention-templates/{template_id}",
    operation_id = "deleteLegalMentionTemplate",
    tag = super::super::TAG,
    params(
        ("template_id" = LegalMentionTemplateId, Path, description = "Template identifier"),
    ),
    responses(
        (status = 204, description = "Template soft-deleted"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
	let current = state
		.usecase
		.get_legal_mention_template(template_id)
		.await?;
	require_org_membership(&state, &identity, current.org_id).await?;
	state
		.usecase
		.soft_delete_legal_mention_template(template_id)
		.await?;

	Ok(Response::NoContent)
}
