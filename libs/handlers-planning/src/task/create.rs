use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    CreateTaskCommand, CustomerContextId, CustomerId, ProjectId, QuoteId, TaskId, TaskStatus,
};
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
    /// How far the work has advanced, declared by the caller and never
    /// derived from the dates — the same register as `blocks_availability`
    /// below. Creating a card straight into `BACKLOG` is the whole point:
    /// a board has no other way to say "agreed, not scheduled".
    ///
    /// Absent means `PLANNED`, so a client written before this field existed
    /// keeps its behavior byte for byte. Absent dates do not imply `BACKLOG`
    /// and a `BACKLOG` status does not clear the dates: "unscheduled" and
    /// "not started" are two separate statements, and only the caller knows
    /// which one it is making.
    #[serde(default)]
    pub status: Option<TaskStatus>,
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
    /// The project this task is costed against. Unrelated to `parent_task_id`:
    /// a subtask may name a project its parent does not.
    #[serde(default)]
    pub project_id: Option<ProjectId>,
    /// What the task costs beyond somebody's time, and why. A non-zero amount
    /// without a label is refused (409): an amount with no reason cannot be
    /// audited later.
    #[serde(default)]
    pub expenses_cents: i32,
    #[serde(default)]
    pub expenses_label: Option<String>,
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
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let task = state
        .usecase
        .acting_as(user_id)
        .create_task(CreateTaskCommand {
            actor,
            organization_id: path.organization_id,
            parent_task_id: payload.parent_task_id,
            title: payload.title,
            description: payload.description,
            starts_at: payload.starts_at,
            ends_at: payload.ends_at,
            all_day: payload.all_day,
            status: payload.status,
            blocks_availability: payload.blocks_availability,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            quote_id: payload.quote_id,
            project_id: payload.project_id,
            expenses_cents: payload.expenses_cents,
            expenses_label: payload.expenses_label,
        })
        .await?;

    Ok(Response::Created(task.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// `title` and `blocks_availability` carry no `#[serde(default)]`, so
    /// every payload below has to name them; everything else is optional.
    fn parse(value: serde_json::Value) -> CreateTaskRequest {
        let mut payload = json!({ "title": "Toiture", "blocks_availability": true });
        let object = payload
            .as_object_mut()
            .expect("the base payload is an object");
        for (key, entry) in value.as_object().expect("the overlay is an object") {
            object.insert(key.clone(), entry.clone());
        }

        serde_json::from_value(payload).expect("payload must deserialize")
    }

    #[test]
    fn an_absent_status_leaves_it_unset_so_the_domain_defaults_to_planned() {
        let request = parse(json!({}));

        assert_eq!(
            request.status, None,
            "an absent `status` must stay `None` all the way to `CreateTaskCommand`, \
             where `TaskService::create_task` turns it into `Planned` — every client \
             written before this field existed keeps its behavior"
        );
    }

    #[test]
    fn the_backlog_wire_form_deserializes_to_the_backlog_status() {
        let request = parse(json!({ "status": "BACKLOG" }));

        assert_eq!(request.status, Some(TaskStatus::Backlog));
    }

    /// The status travels independently of the dates in both directions, at
    /// the wire level as much as in the domain: naming `BACKLOG` alongside a
    /// window is a legal payload, and so is omitting the window entirely.
    #[test]
    fn a_declared_status_and_the_dates_do_not_constrain_each_other() {
        let scheduled_backlog = parse(json!({
            "status": "BACKLOG",
            "starts_at": "2026-09-14T08:00:00Z",
            "ends_at": "2026-09-14T10:00:00Z",
        }));

        assert_eq!(scheduled_backlog.status, Some(TaskStatus::Backlog));
        assert!(scheduled_backlog.starts_at.is_some());

        let undated_planned = parse(json!({ "status": "PLANNED" }));

        assert_eq!(undated_planned.status, Some(TaskStatus::Planned));
        assert_eq!(undated_planned.starts_at, None);
        assert_eq!(undated_planned.ends_at, None);
    }

    /// The generated OpenAPI surface is what the frontend client is built
    /// from, so "the field deserializes" is only half the story — if utoipa
    /// did not pick it up, a generated client would have no way to send it.
    /// `TaskStatus` derives `ToSchema`, so `Option<TaskStatus>` needs no
    /// `#[schema(...)]` annotation here; this pins that, and pins that
    /// `BACKLOG` reaches the published enum.
    #[test]
    fn the_openapi_schema_exposes_status_as_an_optional_task_status() {
        let schema = serde_json::to_value(<CreateTaskRequest as utoipa::PartialSchema>::schema())
            .expect("the generated schema serializes");

        let status = schema
            .pointer("/properties/status")
            .expect("`status` must appear in the generated `CreateTaskRequest` schema");

        assert!(
            !schema
                .pointer("/required")
                .and_then(|required| required.as_array())
                .is_some_and(|required| required.iter().any(|field| field == "status")),
            "`status` must stay optional: a client that omits it gets `PLANNED`"
        );
        assert!(
            serde_json::to_string(status)
                .expect("the property serializes")
                .contains("TaskStatus"),
            "`status` must reference the shared `TaskStatus` schema rather than \
             inlining an ad-hoc enum that could drift from the domain: {status}"
        );
    }

    #[test]
    fn an_unknown_status_is_rejected_rather_than_defaulted() {
        let payload = json!({
            "title": "Toiture",
            "blocks_availability": true,
            "status": "SCHEDULED",
        });

        assert!(
            serde_json::from_value::<CreateTaskRequest>(payload).is_err(),
            "an unrecognized status must be a 400, never a silent fallback to `Planned`"
        );
    }
}
