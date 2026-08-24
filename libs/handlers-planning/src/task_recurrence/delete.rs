use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::task_recurrence::{TaskRecurrencePath, require_recurrence_target};

#[utoipa::path(
    delete,
    path = "/api/v1/task-recurrences/{task_recurrence_id}",
    operation_id = "deleteTaskRecurrence",
    tag = super::super::TAG,
    params(
        ("task_recurrence_id" = mestier_core::TaskRecurrenceId, Path, description = "Task recurrence identifier"),
    ),
    responses(
        (status = 204, description = "Recurrence and its future occurrences soft-deleted; past occurrences are left standing"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Recurrence not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskRecurrencePath { task_recurrence_id }: TaskRecurrencePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_recurrence_target(&state, &identity, task_recurrence_id).await?;

    state
        .usecase
        .delete_task_recurrence(task_recurrence_id)
        .await?;

    Ok(Response::NoContent)
}
