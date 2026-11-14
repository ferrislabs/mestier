use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::SurveyResponseCommand;

use crate::{require_org_membership, response::SurveyResponsePath};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/survey-responses/{response_id}",
    operation_id = "deleteSurveyResponse",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("response_id" = mestier_core::ResponseId, Path, description = "Response identifier"),
    ),
    responses(
        (status = 204, description = "Response deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Response not found, or surveys not enabled"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SurveyResponsePath {
        organization_id,
        response_id,
    }: SurveyResponsePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_survey_response(SurveyResponseCommand {
            actor,
            organization_id,
            id: response_id,
        })
        .await?;

    Ok(Response::NoContent)
}
