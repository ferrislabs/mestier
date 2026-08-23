use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{NaiveDate, NaiveTime};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    CreateTaskRecurrenceCommand, CustomerContextId, CustomerId, MemberId, ProjectId,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership, response::TaskRecurrenceResponse, task::require_task_targets,
    task_recurrence::TaskRecurrencesPath,
};

/// Mirrors `response::RecurrenceRuleResponse`'s wire shape — `frequency`
/// discriminates, only the field the chosen frequency needs is present.
/// Kept as its own request type rather than reusing the response one:
/// `Deserialize` and `Serialize` derive independently, and a request has no
/// business carrying a `ToSchema` meant for a response's documentation.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "frequency", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecurrenceRuleRequest {
    Daily,
    /// ISO weekday numbers, 1 (Monday) through 7 (Sunday). Never empty — a
    /// weekly recurrence with no weekday would produce nothing.
    Weekly {
        weekdays: Vec<i16>,
    },
    /// 1 through 31.
    Monthly {
        day_of_month: u8,
    },
}

impl TryFrom<RecurrenceRuleRequest> for mestier_core::RecurrenceRule {
    type Error = ApiError;

    fn try_from(value: RecurrenceRuleRequest) -> Result<Self, Self::Error> {
        Ok(match value {
            RecurrenceRuleRequest::Daily => Self::Daily,
            RecurrenceRuleRequest::Weekly { weekdays } => Self::Weekly {
                weekdays: weekdays
                    .into_iter()
                    .map(mestier_core::weekday_from_iso)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(ApiError::BadRequest)?,
            },
            RecurrenceRuleRequest::Monthly { day_of_month } => Self::Monthly { day_of_month },
        })
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRecurrenceRequest {
    #[serde(flatten)]
    pub rule: RecurrenceRuleRequest,
    pub starts_on: NaiveDate,
    #[serde(default)]
    pub ends_on: Option<NaiveDate>,
    /// An IANA zone name, e.g. `Europe/Paris`.
    pub timezone: String,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    #[serde(default)]
    pub all_day: bool,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub blocks_availability: bool,
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    #[serde(default)]
    pub customer_context_id: Option<CustomerContextId>,
    #[serde(default)]
    pub project_id: Option<ProjectId>,
    #[serde(default)]
    pub assignee_member_ids: Vec<MemberId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/task-recurrences",
    operation_id = "createTaskRecurrence",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateTaskRecurrenceRequest,
    responses(
        (status = 201, description = "Recurrence created and materialized up to the horizon", body = inline(DataEnvelope<TaskRecurrenceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Assignee, customer, customer context or project not found"),
        (status = 409, description = "Recurrence conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: TaskRecurrencesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateTaskRecurrenceRequest>,
) -> Result<Response<TaskRecurrenceResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    require_task_targets(
        &state,
        path.organization_id,
        payload.customer_id,
        payload.customer_context_id,
        None,
    )
    .await?;

    let timezone: chrono_tz::Tz = payload
        .timezone
        .parse()
        .map_err(|_| ApiError::BadRequest(format!("unknown timezone `{}`", payload.timezone)))?;

    let recurrence = state
        .usecase
        .create_task_recurrence(CreateTaskRecurrenceCommand {
            organization_id: path.organization_id,
            rule: payload.rule.try_into()?,
            starts_on: payload.starts_on,
            ends_on: payload.ends_on,
            timezone,
            start_time: payload.start_time,
            duration_minutes: payload.duration_minutes,
            all_day: payload.all_day,
            title: payload.title,
            description: payload.description,
            blocks_availability: payload.blocks_availability,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            project_id: payload.project_id,
            assignee_member_ids: payload.assignee_member_ids,
        })
        .await?;

    Ok(Response::Created(recurrence.into()))
}
