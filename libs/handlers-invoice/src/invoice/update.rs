use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{
    CustomerContextId, CustomerId, InvoiceId, OperationNature, OrganizationAddress, ProjectId,
    UpdateInvoiceCommand,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    invoice::create::{DeliveryAddressRequest, InvoiceLineRequest, into_line_commands},
    paths::InvoicePath,
    require_invoice_membership, require_invoice_project, require_invoice_targets,
    response::InvoiceResponse,
};

/// `PATCH`, drafts only — `update_invoice` goes through
/// `DraftInvoice::try_from_invoice` and refuses anything else with a
/// `CoreError::Conflict` naming the invoice's actual status, which already
/// arrives here as `ApiError::Conflict` (see `From<CoreError> for ApiError`)
/// with no translation needed.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateInvoiceRequest {
    pub project_id: Option<ProjectId>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub operation_nature: Option<OperationNature>,
    pub delivery_address: Option<DeliveryAddressRequest>,
    pub lines: Vec<InvoiceLineRequest>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/invoices/{invoice_id}",
    operation_id = "updateInvoice",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    request_body = UpdateInvoiceRequest,
    responses(
        (status = 200, description = "Invoice updated", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "Invoice is not a draft and cannot be edited"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoicePath { invoice_id }: InvoicePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    let current = require_invoice_membership(&state, &identity, invoice_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;
    require_invoice_targets(
        &state,
        current.organization_id,
        payload.customer_id,
        payload.customer_context_id,
    )
    .await?;
    if let Some(project_id) = payload.project_id {
        require_invoice_project(&state, current.organization_id, project_id).await?;
    }

    let invoice = state
        .usecase
        .acting_as(actor)
        .update_invoice(UpdateInvoiceCommand {
            id: invoice_id,
            project_id: payload.project_id,
            customer_id: payload.customer_id,
            customer_context_id: payload.customer_context_id,
            due_at: payload.due_at,
            notes: payload.notes,
            operation_nature: payload.operation_nature,
            delivery_address: payload.delivery_address.map(OrganizationAddress::from),
            lines: into_line_commands(payload.lines)?,
        })
        .await?;

    Ok(Response::OK(invoice.into()))
}
