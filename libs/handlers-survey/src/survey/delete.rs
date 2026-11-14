use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::SurveyCommand;

use crate::{require_org_membership, survey::SurveyPath};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/surveys/{survey_id}",
    operation_id = "deleteSurvey",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    responses(
        (status = 204, description = "Survey deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Survey not found, or surveys not enabled"),
        (status = 409, description = "The survey has responses: close it instead"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SurveyPath {
        organization_id,
        survey_id,
    }: SurveyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_survey(SurveyCommand {
            actor,
            organization_id,
            id: survey_id,
        })
        .await?;

    Ok(Response::NoContent)
}
