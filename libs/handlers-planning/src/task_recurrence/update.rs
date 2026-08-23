use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{NaiveDate, NaiveTime};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{MemberId, PatchTaskRecurrenceCommand, ProjectId};
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

use crate::{
    response::TaskRecurrenceResponse,
    task_recurrence::{TaskRecurrencePath, require_recurrence_target},
};

use super::create::RecurrenceRuleRequest;

/// Same "absent leaves it, present (`null` included) applies it" distinction
/// as `task::update::deserialize_present` — duplicated rather than shared
/// across modules, matching that module's own choice.
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

/// Every field is optional and, when present, replaces the current value.
/// `ends_on`, `description` and `project_id` additionally distinguish
/// "absent" from "present but `null`". Changing `rule`, `start_time`,
/// `duration_minutes` or the template fields only affects occurrences
/// materialized after this call — see `TaskRecurrence`'s own doc.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskRecurrenceRequest {
    #[serde(flatten)]
    pub rule: Option<RecurrenceRuleRequest>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<NaiveDate>, nullable)]
    pub ends_on: Option<Option<NaiveDate>>,
    #[serde(default)]
    pub start_time: Option<NaiveTime>,
    #[serde(default)]
    pub duration_minutes: Option<i32>,
    #[serde(default)]
    pub all_day: Option<bool>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub blocks_availability: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<ProjectId>, nullable)]
    pub project_id: Option<Option<ProjectId>>,
    #[serde(default)]
    pub assignee_member_ids: Option<Vec<MemberId>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/task-recurrences/{task_recurrence_id}",
    operation_id = "patchTaskRecurrence",
    tag = super::super::TAG,
    params(
        ("task_recurrence_id" = mestier_core::TaskRecurrenceId, Path, description = "Task recurrence identifier"),
    ),
    request_body = UpdateTaskRecurrenceRequest,
    responses(
        (status = 200, description = "Recurrence rule/template updated — already-materialized occurrences are untouched", body = inline(DataEnvelope<TaskRecurrenceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Recurrence or assignee not found"),
        (status = 409, description = "Recurrence conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskRecurrencePath { task_recurrence_id }: TaskRecurrencePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateTaskRecurrenceRequest>,
) -> Result<Response<TaskRecurrenceResponse>, ApiError> {
    require_recurrence_target(&state, &identity, task_recurrence_id).await?;

    let rule = payload.rule.map(TryInto::try_into).transpose()?;

    let recurrence = state
        .usecase
        .patch_task_recurrence(PatchTaskRecurrenceCommand {
            id: task_recurrence_id,
            rule,
            ends_on: payload.ends_on,
            start_time: payload.start_time,
            duration_minutes: payload.duration_minutes,
            all_day: payload.all_day,
            title: payload.title,
            description: payload.description,
            blocks_availability: payload.blocks_availability,
            project_id: payload.project_id,
            assignee_member_ids: payload.assignee_member_ids,
        })
        .await?;

    Ok(Response::OK(recurrence.into()))
}
