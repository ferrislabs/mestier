use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ArchiveScope;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::WorkflowsPath, require_view_automation, response::WorkflowResponse};

#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListWorkflowsQuery {
    #[serde(default)]
    pub archived: bool,
}

impl ListWorkflowsQuery {
    fn scope(&self) -> ArchiveScope {
        if self.archived {
            ArchiveScope::Archived
        } else {
            ArchiveScope::Active
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/workflows",
    operation_id = "listWorkflows",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListWorkflowsQuery,
    ),
    responses(
        (status = 200, description = "Workflows for this organization, archived ones excluded unless `archived=true`, in which case only archived ones", body = inline(DataEnvelope<Vec<WorkflowResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: WorkflowsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(filter): Query<ListWorkflowsQuery>,
) -> Result<Response<Vec<WorkflowResponse>>, ApiError> {
    require_view_automation(&state, &identity, path.organization_id).await?;

    let workflows = state
        .usecase
        .list_workflows_by_scope(path.organization_id, filter.scope())
        .await?;
    let body: Vec<WorkflowResponse> = workflows.into_iter().map(WorkflowResponse::from).collect();

    Ok(Response::OK(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_query_asks_for_active_workflows_only() {
        assert_eq!(ListWorkflowsQuery::default().scope(), ArchiveScope::Active);
    }

    #[test]
    fn archived_true_asks_for_archived_workflows_only() {
        let query: ListWorkflowsQuery =
            serde_json::from_value(serde_json::json!({ "archived": true })).unwrap();

        assert_eq!(query.scope(), ArchiveScope::Archived);
    }

    #[test]
    fn archived_false_is_the_default() {
        let query: ListWorkflowsQuery =
            serde_json::from_value(serde_json::json!({ "archived": false })).unwrap();

        assert_eq!(query.scope(), ArchiveScope::Active);
    }
}
