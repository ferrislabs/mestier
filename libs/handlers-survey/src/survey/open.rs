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
    survey::{OpenSurveyPath, SurveyView},
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/surveys/{survey_id}/open",
    operation_id = "openSurvey",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    responses(
        (status = 200, description = "Survey opened: its definition is frozen and survey.opened is emitted", body = inline(DataEnvelope<SurveyView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Survey not found, or surveys not enabled"),
        (status = 409, description = "The survey is not a draft, has no graph, or changed meanwhile"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    OpenSurveyPath {
        organization_id,
        survey_id,
    }: OpenSurveyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let survey = state
        .usecase
        .acting_as(user_id)
        .open_survey(SurveyCommand {
            actor,
            organization_id,
            id: survey_id,
        })
        .await?;

    Ok(Response::OK(SurveyView::opened(&survey)).into_response())
}
