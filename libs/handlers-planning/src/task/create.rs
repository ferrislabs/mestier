use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{CreateTaskCommand, CustomerContextId, CustomerId, QuoteId, TaskId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    response::TaskResponse,
    task::{TasksPath, require_task_targets},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    #[serde(default)]
    pub parent_task_id: Option<TaskId>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub all_day: bool,
    /// Declared here, never guessed: whether this task makes its assignees
    /// unavailable elsewhere. The caller states it explicitly on every
    /// creation — see the planning module design doc's invariant 9.
    pub blocks_availability: bool,
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    #[serde(default)]
    pub customer_context_id: Option<CustomerContextId>,
    #[serde(default)]
    pub quote_id: Option<QuoteId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/tasks",
    operation_id = "createTask",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = inline(DataEnvelope<TaskResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Task conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TasksPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Response<TaskResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    require_task_targets(
        &state,
        path.organization_id,
        payload.customer_id,
        payload.customer_context_id,
        payload.quote_id,
    )
    .await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let task = state
        .usecase
        .acting_as(actor)
        .create_task(CreateTaskCommand {
            organization_id: path.organization_id,
            parent_task_id: payload.parent_task_id,
            title: payload.title,
            description: payload.description,
            starts_at: payload.starts_at,
            ends_at: payload.ends_at,
            all_day: payload.all_day,
            blocks_availability: payload.blocks_availability,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            quote_id: payload.quote_id,
        })
        .await?;

    Ok(Response::Created(task.into()))
}
