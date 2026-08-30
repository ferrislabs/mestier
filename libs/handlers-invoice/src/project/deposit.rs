use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{IssueDepositCommand, ProjectId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::ProjectInvoiceDepositPath, require_project_membership, response::InvoiceResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct IssueDepositRequest {
    /// Basis points of the quote's net total, e.g. 3000 = 30%.
    pub percentage_bp: i32,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    #[serde(default)]
    pub allow_exceeding_total: bool,
}

/// Builds a deposit line for `percentage_bp` of the project's quoted net
/// total and issues it in one call — `IssueDepositCommand` (#317), reachable
/// over HTTP for the first time here. Not in the master issue's own route
/// list: added because leaving it unreachable would make #322 impossible to
/// build later.
#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_id}/invoices/deposit",
    operation_id = "issueProjectDeposit",
    tag = super::super::TAG,
    params(
        ("project_id" = ProjectId, Path, description = "Project identifier"),
    ),
    request_body = IssueDepositRequest,
    responses(
        (status = 201, description = "Deposit invoice issued", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "The project has no quote, no customer, or the deposit would exceed the quoted total"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectInvoiceDepositPath { project_id }: ProjectInvoiceDepositPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<IssueDepositRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    require_project_membership(&state, &identity, project_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let invoice = state
        .usecase
        .acting_as(user_id)
        .issue_deposit(IssueDepositCommand {
            project_id,
            actor,
            percentage_bp: payload.percentage_bp,
            due_at: payload.due_at,
            notes: payload.notes,
            allow_exceeding_total: payload.allow_exceeding_total,
        })
        .await?;

    Ok(Response::Created(invoice.into()))
}
