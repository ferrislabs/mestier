use auth::Identity;
use axum::{
    Extension,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::SurveyResponseCommand;

use crate::{
    require_org_membership,
    response::{RespondentResponseView, member::MemberResponseCompletionPath},
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/member-survey-responses/{response_id}/complete",
    operation_id = "completeMemberSurveyResponse",
    tag = super::super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("response_id" = mestier_core::ResponseId, Path, description = "Response identifier"),
    ),
    responses(
        (status = 200, description = "Response completed; survey.response_completed is emitted once", body = inline(DataEnvelope<RespondentResponseView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of the organization"),
        (status = 404, description = "No response of this member with this id, or surveys not enabled"),
        (status = 409, description = "Closed, outside its campaign window, response cap reached, already complete or already answered"),
        (status = 422, description = "A required question on the path is unanswered"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberResponseCompletionPath {
        organization_id,
        response_id,
    }: MemberResponseCompletionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let response = state
        .usecase
        .acting_as(user_id)
        .complete_member_response(SurveyResponseCommand {
            actor,
            organization_id,
            id: response_id,
        })
        .await?;

    Ok(Response::OK(RespondentResponseView::from(&response)).into_response())
}
