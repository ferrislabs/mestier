use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{IssueFinalInvoiceCommand, ProjectId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::ProjectInvoiceFinalPath, require_project_membership, response::InvoiceResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct IssueFinalInvoiceRequest {
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    #[serde(default)]
    pub allow_exceeding_total: bool,
}

/// Builds an invoice for exactly what remains on the project's quote and
/// issues it in one call — `IssueFinalInvoiceCommand` (#317), reachable
/// over HTTP for the first time here. Same "unreachable domain capability"
/// reasoning as `project::deposit`.
#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_id}/invoices/final",
    operation_id = "issueProjectFinalInvoice",
    tag = super::super::TAG,
    params(
        ("project_id" = ProjectId, Path, description = "Project identifier"),
    ),
    request_body = IssueFinalInvoiceRequest,
    responses(
        (status = 201, description = "Final invoice issued", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "The project has no quote, no customer, or nothing left to bill"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProjectInvoiceFinalPath { project_id }: ProjectInvoiceFinalPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<IssueFinalInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    require_project_membership(&state, &identity, project_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let invoice = state
        .usecase
        .acting_as(user_id)
        .issue_final_invoice(IssueFinalInvoiceCommand {
            project_id,
            actor,
            due_at: payload.due_at,
            notes: payload.notes,
            allow_exceeding_total: payload.allow_exceeding_total,
        })
        .await?;

    Ok(Response::Created(invoice.into()))
}
