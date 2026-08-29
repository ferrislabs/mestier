use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::DeleteScope;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::task::{TaskPath, require_task};

/// How much of a series a `DELETE` removes. Absent (a task that never
/// belonged to one, or the caller not asking) behaves like `THIS_OCCURRENCE`
/// — the only meaningful choice when there is no series to reach into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeleteScopeRequest {
    ThisOccurrence,
    ThisAndFollowing,
}

impl From<DeleteScopeRequest> for DeleteScope {
    fn from(value: DeleteScopeRequest) -> Self {
        match value {
            DeleteScopeRequest::ThisOccurrence => DeleteScope::ThisOccurrence,
            DeleteScopeRequest::ThisAndFollowing => DeleteScope::ThisAndFollowing,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeleteTaskQuery {
    #[serde(default)]
    pub scope: Option<DeleteScopeRequest>,
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}",
    operation_id = "deleteTask",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
        DeleteTaskQuery,
    ),
    responses(
        (status = 204, description = "Task soft-deleted — `?scope=this_and_following` also removes every later occurrence in the same series"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Task not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskPath {
        organization_id,
        task_id,
    }: TaskPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<DeleteTaskQuery>,
) -> Result<Response<()>, ApiError> {
    require_task(&state, &identity, organization_id, task_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let scope = query
        .scope
        .map(DeleteScope::from)
        .unwrap_or(DeleteScope::ThisOccurrence);
    state
        .usecase
        .acting_as(user_id)
        .soft_delete_task_occurrence(actor, task_id, scope)
        .await?;

    Ok(Response::NoContent)
}
