use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::TaskId;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{require_org_membership, response::TaskResponse, task::TasksPath};

/// `?parent_task_id=` scopes the listing to a specific task's children;
/// absent, it lists roots — see `GET /tasks`'s contract.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTasksQuery {
    #[serde(default)]
    pub parent_task_id: Option<TaskId>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/tasks",
    operation_id = "listTasks",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
        ListTasksQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of tasks — each root's child_count is populated, computed without loading its subtasks", body = inline(DataEnvelope<Vec<TaskResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TasksPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<ListTasksQuery>,
) -> Result<Response<TaskResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (tasks, child_counts, total) = state
        .usecase
        .list_tasks(
            path.organization_id,
            filter.parent_task_id,
            per_page,
            offset,
        )
        .await?;
    let items: Vec<TaskResponse> = tasks
        .into_iter()
        .map(|task| {
            let child_count = child_counts.get(&task.id).copied().unwrap_or(0);
            TaskResponse {
                child_count: Some(child_count),
                ..TaskResponse::from(task)
            }
        })
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
