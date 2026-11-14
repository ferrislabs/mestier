use auth::Identity;
use axum::{
    Extension,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::SurveyCommand;

use crate::{
    require_org_membership,
    survey::{DuplicateSurveyPath, SurveyView},
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/surveys/{survey_id}/duplicate",
    operation_id = "duplicateSurvey",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    responses(
        (status = 201, description = "Draft copy of the survey created", body = inline(DataEnvelope<SurveyView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Survey not found, or surveys not enabled"),
        (status = 409, description = "The survey changed meanwhile"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DuplicateSurveyPath {
        organization_id,
        survey_id,
    }: DuplicateSurveyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let survey = state
        .usecase
        .acting_as(user_id)
        .duplicate_survey(SurveyCommand {
            actor,
            organization_id,
            id: survey_id,
        })
        .await?;

    Ok(Response::Created(SurveyView::draft(&survey)).into_response())
}
