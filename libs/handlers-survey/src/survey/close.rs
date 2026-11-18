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
    survey::{CloseSurveyPath, SurveyView},
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/surveys/{survey_id}/close",
    operation_id = "closeSurvey",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    responses(
        (status = 200, description = "Survey closed", body = inline(DataEnvelope<SurveyView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Survey not found, or surveys not enabled"),
        (status = 409, description = "The survey is not opened, or changed meanwhile"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CloseSurveyPath {
        organization_id,
        survey_id,
    }: CloseSurveyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let survey = state
        .usecase
        .acting_as(user_id)
        .close_survey(SurveyCommand {
            actor,
            organization_id,
            id: survey_id,
        })
        .await?;

    Ok(Response::OK(SurveyView::closed(&survey)).into_response())
}
