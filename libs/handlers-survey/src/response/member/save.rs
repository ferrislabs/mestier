use auth::Identity;
use axum::{
    Extension, Json,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::SaveMemberResponseCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    request::answers,
    require_org_membership,
    response::{RespondentResponseView, member::MemberResponsePath},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveSurveyAnswersRequest {
    #[schema(value_type = std::collections::BTreeMap<String, crate::schema::SurveyAnswerSchema>)]
    pub answers: serde_json::Value,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/member-survey-responses/{response_id}",
    operation_id = "saveMemberSurveyResponse",
    tag = super::super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("response_id" = mestier_core::ResponseId, Path, description = "Response identifier"),
    ),
    request_body(
        content = SaveSurveyAnswersRequest,
        description = "Every answer given so far, keyed by question id, in the domain form of an answer. Answers off the path and blank answers are dropped.",
        example = json!({
            "answers": {
                "0195f0a0-0000-7000-8000-000000000001": {"Choice": "Non"},
                "0195f0a0-0000-7000-8000-000000000003": {"Text": "Trop court"}
            }
        })
    ),
    responses(
        (status = 200, description = "Answers saved; survey.response_submitted is emitted", body = inline(DataEnvelope<RespondentResponseView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of the organization"),
        (status = 404, description = "No response of this member with this id, or surveys not enabled"),
        (status = 409, description = "Closed, outside its campaign window, or already complete"),
        (status = 422, description = "An answer names an unknown question or option, does not match its kind, or is an invalid email, phone or date"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberResponsePath {
        organization_id,
        response_id,
    }: MemberResponsePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SaveSurveyAnswersRequest>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let answers = answers(payload.answers)?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let response = state
        .usecase
        .acting_as(user_id)
        .save_member_response(SaveMemberResponseCommand {
            actor,
            organization_id,
            id: response_id,
            answers,
        })
        .await?;

    Ok(Response::OK(RespondentResponseView::from(&response)).into_response())
}
