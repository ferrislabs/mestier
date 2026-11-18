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
    response::{RespondentSurveyView, member::MemberSurveyPath},
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/member-surveys/{survey_id}",
    operation_id = "openMemberSurvey",
    tag = super::super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    responses(
        (status = 200, description = "The opened members survey as its respondent sees it: no organization, metadata, settings nor campaign", body = inline(DataEnvelope<RespondentSurveyView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of the organization"),
        (status = 404, description = "No opened members survey with this id, or surveys not enabled"),
        (status = 409, description = "The survey is closed"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberSurveyPath {
        organization_id,
        survey_id,
    }: MemberSurveyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let survey = state
        .usecase
        .acting_as(user_id)
        .open_member_survey(SurveyCommand {
            actor,
            organization_id,
            id: survey_id,
        })
        .await?;

    Ok(Response::OK(RespondentSurveyView::from(&survey)).into_response())
}
