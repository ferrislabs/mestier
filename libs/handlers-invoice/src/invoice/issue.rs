use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{InvoiceId, IssueInvoiceCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::InvoiceIssuePath, require_invoice_membership, response::InvoiceResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct IssueInvoiceRequest {
    #[serde(default)]
    pub allow_exceeding_total: bool,
}

#[utoipa::path(
    post,
    path = "/api/v1/invoices/{invoice_id}/issue",
    operation_id = "issueInvoice",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    request_body = IssueInvoiceRequest,
    responses(
        (status = 200, description = "Invoice issued", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "Invoice is not a draft, or issuing it would exceed the project's quoted total"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceIssuePath { invoice_id }: InvoiceIssuePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<IssueInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let invoice = state
        .usecase
        .acting_as(user_id)
        .issue_invoice(IssueInvoiceCommand {
            id: invoice_id,
            actor,
            allow_exceeding_total: payload.allow_exceeding_total,
        })
        .await?;

    Ok(Response::OK(invoice.into()))
}
