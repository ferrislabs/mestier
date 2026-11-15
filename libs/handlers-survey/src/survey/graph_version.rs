use auth::Identity;
use axum::{
    Extension,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{SurveyGraph, SurveyGraphVersion, SurveyGraphVersionCommand};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{require_org_membership, survey::SurveyGraphVersionPath};

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct SurveyGraphVersionView<'a> {
    #[schema(minimum = 1)]
    pub version: u32,
    #[schema(value_type = crate::schema::SurveyGraphSchema)]
    pub graph: &'a SurveyGraph,
}

impl<'a> From<&'a SurveyGraphVersion> for SurveyGraphVersionView<'a> {
    fn from(stored: &'a SurveyGraphVersion) -> Self {
        Self {
            version: stored.version(),
            graph: stored.graph(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/surveys/{survey_id}/graph-versions/{version}",
    operation_id = "getSurveyGraphVersion",
    tag = super::super::TAG,
    description = "Reads the graph a completed response was given on: the current graph when the version is the current one, otherwise the version kept when the graph changed after a completed response.",
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
        ("version" = u32, Path, description = "Graph version, the `graph_version` of a completed response"),
    ),
    responses(
        (status = 200, description = "The graph of that version", body = inline(DataEnvelope<SurveyGraphVersionView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.view or survey.manage"),
        (status = 404, description = "Survey or version not found, or surveys not enabled"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SurveyGraphVersionPath {
        organization_id,
        survey_id,
        version,
    }: SurveyGraphVersionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let stored = state
        .usecase
        .acting_as(user_id)
        .get_survey_graph_version(SurveyGraphVersionCommand {
            actor,
            organization_id,
            id: survey_id,
            version,
        })
        .await?;

    Ok(Response::OK(SurveyGraphVersionView::from(&stored)).into_response())
}
