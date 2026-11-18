use auth::Identity;
use axum::{
    Extension,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::StartMemberResponseCommand;

use crate::{
    require_org_membership,
    response::{RespondentResponseView, member::MemberSurveyResponsesPath},
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/member-surveys/{survey_id}/responses",
    operation_id = "startMemberSurveyResponse",
    tag = super::super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    responses(
        (status = 200, description = "The member's response in progress resumed", body = inline(DataEnvelope<RespondentResponseView>)),
        (status = 201, description = "Response started", body = inline(DataEnvelope<RespondentResponseView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of the organization"),
        (status = 404, description = "No opened members survey with this id, or surveys not enabled"),
        (status = 409, description = "Closed, outside its campaign window, response cap reached, or already answered"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberSurveyResponsesPath {
        organization_id,
        survey_id,
    }: MemberSurveyResponsesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let started = state
        .usecase
        .acting_as(user_id)
        .start_member_response(StartMemberResponseCommand {
            actor,
            organization_id,
            survey_id,
        })
        .await?;

    let view = RespondentResponseView::from(started.response());
    if started.is_resumed() {
        Ok(Response::OK(view).into_response())
    } else {
        Ok(Response::Created(view).into_response())
    }
}
