use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    CreateQuoteCommand, CustomerContextId, CustomerId, QuoteLineCommand, ServiceRateId,
    ServiceRateUnit,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::QuotesPath, require_org_membership, require_quote_targets, response::QuoteResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct QuoteLineRequest {
    pub service_rate_id: Option<ServiceRateId>,
    pub label: String,
    pub quantity: String,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    #[serde(default)]
    pub vat_rate_bp: Option<i32>,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
}

impl TryFrom<QuoteLineRequest> for QuoteLineCommand {
    type Error = ApiError;

    fn try_from(value: QuoteLineRequest) -> Result<Self, Self::Error> {
        let quantity = Decimal::from_str(&value.quantity)
            .map_err(|_| ApiError::Validation("quote line quantity must be decimal".to_owned()))?;

        Ok(Self {
            service_rate_id: value.service_rate_id,
            label: value.label,
            quantity,
            unit: value.unit,
            unit_price_cents: value.unit_price_cents,
            vat_rate_bp: value.vat_rate_bp,
            notes: value.notes,
            photo_keys: value.photo_keys,
        })
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateQuoteRequest {
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub lines: Vec<QuoteLineRequest>,
}

pub fn into_line_commands(lines: Vec<QuoteLineRequest>) -> Result<Vec<QuoteLineCommand>, ApiError> {
    lines.into_iter().map(TryInto::try_into).collect()
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/quotes",
    operation_id = "createQuote",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateQuoteRequest,
    responses(
        (status = 201, description = "Quote created", body = inline(DataEnvelope<QuoteResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Quote conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: QuotesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateQuoteRequest>,
) -> Result<Response<QuoteResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    require_quote_targets(
        &state,
        path.organization_id,
        payload.customer_id,
        payload.customer_context_id,
    )
    .await?;

    let quote = state
        .usecase
        .acting_as(user_id)
        .create_quote(
            CreateQuoteCommand {
                organization_id: path.organization_id,
                title: payload.title,
                customer_id: payload.customer_id,
                customer_context_id: payload.customer_context_id,
                lines: into_line_commands(payload.lines)?,
            },
            actor,
        )
        .await?;

    Ok(Response::Created(quote.into()))
}
