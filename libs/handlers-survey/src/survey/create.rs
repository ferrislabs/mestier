use auth::Identity;
use axum::{
    Extension, Json,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateSurveyCommand, Visibility};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    survey::{SurveyView, SurveysPath},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSurveyRequest {
    pub title: String,
    pub visibility: Visibility,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/surveys",
    operation_id = "createSurvey",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateSurveyRequest,
    responses(
        (status = 201, description = "Draft survey created", body = inline(DataEnvelope<SurveyView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Surveys are not enabled for this organization"),
        (status = 422, description = "Blank title"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: SurveysPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateSurveyRequest>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let survey = state
        .usecase
        .acting_as(user_id)
        .create_survey(CreateSurveyCommand {
            actor,
            organization_id: path.organization_id,
            title: payload.title,
            visibility: payload.visibility,
        })
        .await?;

    Ok(Response::Created(SurveyView::draft(&survey)).into_response())
}
