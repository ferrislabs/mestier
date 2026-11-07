use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    BillingFrequency, CreateQuoteCommand, CustomerContactId, CustomerContextId, CustomerId, DealId,
    MemberId, ProductId, QuoteLineCommand, QuoteTermsCommand, ServiceRateId,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::QuotesPath, quote_error::QuoteFailure, require_org_membership, require_quote_deal,
    require_quote_targets, response::QuoteResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct QuoteLineRequest {
    pub service_rate_id: Option<ServiceRateId>,
    #[serde(default)]
    pub product_id: Option<ProductId>,
    pub label: String,
    pub quantity: String,
    /// One of the built-in `ServiceRateUnit` codes, or an organization's own
    /// custom unit code (#449).
    pub unit: String,
    pub unit_price_cents: i32,
    #[serde(default)]
    pub vat_rate_bp: Option<i32>,
    #[serde(default)]
    pub discount_percent: Option<String>,
    #[serde(default)]
    pub billing_frequency: Option<BillingFrequency>,
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
            product_id: value.product_id,
            label: value.label,
            quantity,
            unit: value.unit,
            unit_price_cents: value.unit_price_cents,
            vat_rate_bp: value.vat_rate_bp,
            discount_percent: parse_percent(value.discount_percent.as_deref(), "discount_percent")?,
            billing_frequency: value.billing_frequency.unwrap_or_default(),
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
    #[serde(default)]
    pub deal_id: Option<DealId>,
    #[serde(default)]
    pub valid_until: Option<NaiveDate>,
    #[serde(default)]
    pub deposit_percent: Option<String>,
    #[serde(default)]
    pub owner_member_id: Option<MemberId>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub billing_contact_id: Option<CustomerContactId>,
    #[serde(default)]
    pub billing_address_id: Option<CustomerContextId>,
    pub lines: Vec<QuoteLineRequest>,
}

pub fn parse_percent(raw: Option<&str>, field: &str) -> Result<Decimal, ApiError> {
    raw.map_or(Ok(Decimal::ZERO), |text| {
        Decimal::from_str(text.trim())
            .map_err(|_| ApiError::Validation(format!("{field} must be decimal")))
    })
}

pub fn into_terms(
    valid_until: Option<NaiveDate>,
    deposit_percent: Option<String>,
    owner_member_id: Option<MemberId>,
    notes: Option<String>,
    billing_contact_id: Option<CustomerContactId>,
    billing_address_id: Option<CustomerContextId>,
) -> Result<QuoteTermsCommand, ApiError> {
    Ok(QuoteTermsCommand {
        valid_until,
        deposit_percent: parse_percent(deposit_percent.as_deref(), "deposit_percent")?,
        owner_member_id,
        notes,
        billing_contact_id,
        billing_address_id,
    })
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
        (status = 422, description = "A rule refused the quote, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: QuotesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateQuoteRequest>,
) -> Result<Response<QuoteResponse>, QuoteFailure> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    require_quote_targets(
        &state,
        path.organization_id,
        payload.customer_id,
        payload.customer_context_id,
    )
    .await?;
    require_quote_deal(&state, path.organization_id, payload.deal_id).await?;

    let terms = into_terms(
        payload.valid_until,
        payload.deposit_percent,
        payload.owner_member_id,
        payload.notes,
        payload.billing_contact_id,
        payload.billing_address_id,
    )?;

    let quote = state
        .usecase
        .acting_as(user_id)
        .create_quote(
            CreateQuoteCommand {
                organization_id: path.organization_id,
                title: payload.title,
                customer_id: payload.customer_id,
                customer_context_id: payload.customer_context_id,
                deal_id: payload.deal_id,
                terms,
                lines: into_line_commands(payload.lines)?,
            },
            actor,
        )
        .await??;

    Ok(Response::Created(quote.into()))
}
