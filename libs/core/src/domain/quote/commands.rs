use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{
    BillingFrequency, CustomerContactId, CustomerContextId, CustomerId, DealId, MemberId,
    OrganizationId, ProductId, QuoteId, QuoteStatus, ServiceRateId,
};

#[derive(Debug, Clone)]
pub struct QuoteLineCommand {
    pub service_rate_id: Option<ServiceRateId>,
    pub product_id: Option<ProductId>,
    pub label: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_price_cents: i32,
    pub vat_rate_bp: Option<i32>,
    pub discount_percent: Decimal,
    pub billing_frequency: BillingFrequency,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct QuoteTermsCommand {
    pub valid_until: Option<NaiveDate>,
    pub deposit_percent: Decimal,
    pub owner_member_id: Option<MemberId>,
    pub notes: Option<String>,
    pub billing_contact_id: Option<CustomerContactId>,
    pub billing_address_id: Option<CustomerContextId>,
}

#[derive(Debug, Clone)]
pub struct CreateQuoteCommand {
    pub organization_id: OrganizationId,
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub deal_id: Option<DealId>,
    pub terms: QuoteTermsCommand,
    pub lines: Vec<QuoteLineCommand>,
}

#[derive(Debug, Clone)]
pub struct UpdateQuoteCommand {
    pub id: QuoteId,
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub deal_id: Option<DealId>,
    pub status: QuoteStatus,
    pub terms: QuoteTermsCommand,
    pub lines: Vec<QuoteLineCommand>,
}

#[derive(Debug, Clone)]
pub struct UpdateQuoteStatusCommand {
    pub id: QuoteId,
    pub status: QuoteStatus,
    pub responded_by: Option<String>,
    pub decline_reason: Option<String>,
}

impl UpdateQuoteStatusCommand {
    pub fn to(id: QuoteId, status: QuoteStatus) -> Self {
        Self {
            id,
            status,
            responded_by: None,
            decline_reason: None,
        }
    }
}
