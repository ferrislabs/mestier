use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_actor,
};
use mestier_core::ListSurveyResponsesCommand;

use crate::{
    require_org_membership,
    response::{ResponseView, SurveyResponsesPath},
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/surveys/{survey_id}/responses",
    operation_id = "listSurveyResponses",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated responses of the survey", body = inline(DataEnvelope<Vec<ResponseView>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.view or survey.manage"),
        (status = 404, description = "Survey not found, or surveys not enabled"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SurveyResponsesPath {
        organization_id,
        survey_id,
    }: SurveyResponsesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let per_page = pagination.per_page();
    let (responses, total) = state
        .usecase
        .acting_as(user_id)
        .list_survey_responses(ListSurveyResponsesCommand {
            actor,
            organization_id,
            survey_id,
            limit: per_page,
            offset: pagination.offset(),
        })
        .await?;
    let items: Vec<ResponseView<'_>> = responses.iter().map(ResponseView::from).collect();
    let meta = PaginationMetadata::new(per_page, pagination.page(), Some(total), items.is_empty());

    Ok(Response::Paginated(Page::new(items, meta)).into_response())
}
