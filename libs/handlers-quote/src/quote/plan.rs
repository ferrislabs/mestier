use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{CreateProjectFromQuoteCommand, PlannedTaskCommand, QuoteLineId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::QuotePlanPath, require_quote_membership, response::QuotePlanResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlannedTaskRequest {
    /// The position (0-based, within `tasks` below) of another task of the
    /// same request, or `null` for a root task.
    #[serde(default)]
    pub parent_index: Option<usize>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    /// `null` on a subtask means "inherit the parent's window" — a root
    /// must carry both, or neither is accepted (see
    /// `ProjectService::build_planned_tasks`).
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub all_day: bool,
    pub blocks_availability: bool,
    #[serde(default)]
    pub expenses_cents: i32,
    #[serde(default)]
    pub expenses_label: Option<String>,
    /// The quote lines this task accounts for — zero or more. Supply lines
    /// and anything else the caller leaves out simply have no task.
    #[serde(default)]
    pub quote_line_ids: Vec<QuoteLineId>,
}

impl From<PlannedTaskRequest> for PlannedTaskCommand {
    fn from(value: PlannedTaskRequest) -> Self {
        Self {
            parent_index: value.parent_index,
            title: value.title,
            description: value.description,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            blocks_availability: value.blocks_availability,
            expenses_cents: value.expenses_cents,
            expenses_label: value.expenses_label,
            quote_line_ids: value.quote_line_ids,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateQuotePlanRequest {
    pub name: String,
    /// A quote already turned into a project refuses a second one unless
    /// this is explicitly `true` — see `validate_quote_plannable`.
    #[serde(default)]
    pub force_new: bool,
    #[serde(default)]
    pub tasks: Vec<PlannedTaskRequest>,
}

/// The write that accepts what a human confirmed: one project (carrying the
/// quote's customer and the quote itself), and the tasks under it. Never
/// triggered by `quote.accepted` automatically — a quote line is a
/// commercial unit, a task is a scheduling unit, and only a person reviewing
/// the proposal can say how the two line up.
#[utoipa::path(
    post,
    path = "/api/v1/quotes/{quote_id}/plan",
    operation_id = "createQuotePlan",
    tag = super::super::TAG,
    params(
        ("quote_id" = mestier_core::QuoteId, Path, description = "Quote identifier"),
    ),
    request_body = CreateQuotePlanRequest,
    responses(
        (status = 201, description = "Project and tasks created from the confirmed plan", body = inline(DataEnvelope<QuotePlanResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
        (status = 409, description = "The quote is not accepted, already has a project, or the plan is invalid"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuotePlanPath { quote_id }: QuotePlanPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateQuotePlanRequest>,
) -> Result<Response<QuotePlanResponse>, ApiError> {
    require_quote_membership(&state, &identity, quote_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let (project, tasks) = state
        .usecase
        .acting_as(actor)
        .create_project_from_quote(CreateProjectFromQuoteCommand {
            quote_id,
            name: payload.name,
            force_new: payload.force_new,
            tasks: payload.tasks.into_iter().map(Into::into).collect(),
        })
        .await?;

    Ok(Response::Created(QuotePlanResponse {
        project: project.into(),
        tasks: tasks.into_iter().map(Into::into).collect(),
    }))
}
