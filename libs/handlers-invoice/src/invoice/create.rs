use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    CreateInvoiceCommand, CustomerContextId, CustomerId, InvoiceKind, InvoiceLineCommand,
    OperationNature, OrganizationAddress, ProjectId,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::OrganizationInvoicesPath, require_invoice_project, require_invoice_targets,
    require_org_membership, response::InvoiceResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeliveryAddressRequest {
    pub line1: String,
    pub line2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

impl From<DeliveryAddressRequest> for OrganizationAddress {
    fn from(value: DeliveryAddressRequest) -> Self {
        Self {
            line1: value.line1,
            line2: value.line2,
            postal_code: value.postal_code,
            city: value.city,
            country: value.country,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InvoiceLineRequest {
    pub label: String,
    pub quantity: String,
    pub unit_price_cents: i32,
    #[serde(default)]
    pub vat_rate_basis_points: Option<i32>,
}

impl TryFrom<InvoiceLineRequest> for InvoiceLineCommand {
    type Error = ApiError;

    fn try_from(value: InvoiceLineRequest) -> Result<Self, Self::Error> {
        let quantity = Decimal::from_str(&value.quantity).map_err(|_| {
            ApiError::Validation("invoice line quantity must be decimal".to_owned())
        })?;

        Ok(Self {
            label: value.label,
            quantity,
            unit_price_cents: value.unit_price_cents,
            vat_rate_basis_points: value.vat_rate_basis_points,
        })
    }
}

pub fn into_line_commands(
    lines: Vec<InvoiceLineRequest>,
) -> Result<Vec<InvoiceLineCommand>, ApiError> {
    lines.into_iter().map(TryInto::try_into).collect()
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInvoiceRequest {
    pub kind: InvoiceKind,
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
    post,
    path = "/api/v1/organizations/{organization_id}/invoices",
    operation_id = "createInvoice",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateInvoiceRequest,
    responses(
        (status = 201, description = "Invoice created", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Invoice conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrganizationInvoicesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;
    require_invoice_targets(
        &state,
        path.organization_id,
        payload.customer_id,
        payload.customer_context_id,
    )
    .await?;
    if let Some(project_id) = payload.project_id {
        require_invoice_project(&state, path.organization_id, project_id).await?;
    }

    let invoice = state
        .usecase
        .acting_as(user_id)
        .create_invoice(CreateInvoiceCommand {
            actor,
            organization_id: path.organization_id,
            kind: payload.kind,
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

    Ok(Response::Created(invoice.into()))
}
