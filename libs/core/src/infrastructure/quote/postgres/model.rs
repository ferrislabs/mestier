use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    BillingFrequency, CustomerContactId, CustomerContextId, CustomerId, DealId, DepositPercent,
    DiscountPercent, IssuerSnapshot, MemberId, OrganizationId, ProductId, Quote, QuoteId,
    QuoteLine, QuoteLineId, QuoteStatus, QuoteVatBreakdownLine, RecipientSnapshot, ServiceRateId,
};

#[derive(Debug, Clone)]
pub struct QuoteRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub reference: Option<String>,
    pub title: String,
    pub customer_id: Uuid,
    pub customer_context_id: Uuid,
    pub deal_id: Option<Uuid>,
    pub status: String,
    pub net_cents: i32,
    pub vat_breakdown: serde_json::Value,
    pub gross_cents: i32,
    pub valid_until: Option<NaiveDate>,
    pub deposit_percent: Decimal,
    pub owner_member_id: Option<Uuid>,
    pub notes: Option<String>,
    pub billing_contact_id: Option<Uuid>,
    pub billing_address_id: Option<Uuid>,
    pub responded_at: Option<DateTime<Utc>>,
    pub responded_by: Option<String>,
    pub decline_reason: Option<String>,
    pub revision_of_id: Option<Uuid>,
    pub superseded_by_id: Option<Uuid>,
    pub issuer_snapshot: Option<serde_json::Value>,
    pub recipient_snapshot: Option<serde_json::Value>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct QuoteLineRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub quote_id: Uuid,
    pub service_rate_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub label: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_price_cents: i32,
    pub vat_rate_bp: Option<i32>,
    pub discount_percent: Decimal,
    pub billing_frequency: String,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl QuoteRow {
    pub fn into_quote(self, lines: Vec<QuoteLine>) -> Result<Quote, CoreError> {
        let status = QuoteStatus::from_str(&self.status)
            .map_err(|e| CoreError::Internal(format!("invalid quote status in database: {e}")))?;
        let vat_breakdown = parse_vat_breakdown(self.vat_breakdown)?;
        let deposit_percent = DepositPercent::try_from(self.deposit_percent).map_err(|e| {
            CoreError::Internal(format!("invalid quote deposit_percent in database: {e}"))
        })?;
        let issuer_snapshot = self
            .issuer_snapshot
            .map(serde_json::from_value::<IssuerSnapshot>)
            .transpose()
            .map_err(|e| {
                CoreError::Internal(format!("invalid quote issuer_snapshot in database: {e}"))
            })?;
        let recipient_snapshot = self
            .recipient_snapshot
            .map(serde_json::from_value::<RecipientSnapshot>)
            .transpose()
            .map_err(|e| {
                CoreError::Internal(format!("invalid quote recipient_snapshot in database: {e}"))
            })?;

        Ok(Quote {
            id: QuoteId(self.id),
            organization_id: OrganizationId(self.org_id),
            reference: self.reference,
            title: self.title,
            customer_id: CustomerId(self.customer_id),
            customer_context_id: CustomerContextId(self.customer_context_id),
            deal_id: self.deal_id.map(DealId),
            status,
            net_cents: self.net_cents,
            vat_breakdown,
            gross_cents: self.gross_cents,
            valid_until: self.valid_until,
            deposit_percent,
            owner_member_id: self.owner_member_id.map(MemberId),
            notes: self.notes,
            billing_contact_id: self.billing_contact_id.map(CustomerContactId),
            billing_address_id: self.billing_address_id.map(CustomerContextId),
            responded_at: self.responded_at,
            responded_by: self.responded_by,
            decline_reason: self.decline_reason,
            revision_of_id: self.revision_of_id.map(QuoteId),
            superseded_by_id: self.superseded_by_id.map(QuoteId),
            issuer_snapshot,
            recipient_snapshot,
            lines,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// The JSONB column round-trips through this shape rather than deriving
/// `Serialize`/`Deserialize` on the domain type directly: the domain type
/// stays free of any hint that it is ever stored as JSON, and this is the
/// one place that decides the wire shape of the column.
#[derive(serde::Serialize, serde::Deserialize)]
struct VatBreakdownLineJson {
    rate_bp: i32,
    vat_cents: i32,
}

fn parse_vat_breakdown(value: serde_json::Value) -> Result<Vec<QuoteVatBreakdownLine>, CoreError> {
    let lines: Vec<VatBreakdownLineJson> = serde_json::from_value(value).map_err(|e| {
        CoreError::Internal(format!("invalid quote vat_breakdown in database: {e}"))
    })?;

    Ok(lines
        .into_iter()
        .map(|line| QuoteVatBreakdownLine {
            rate_bp: line.rate_bp,
            vat_cents: line.vat_cents,
        })
        .collect())
}

pub fn vat_breakdown_to_json(breakdown: &[QuoteVatBreakdownLine]) -> serde_json::Value {
    serde_json::to_value(
        breakdown
            .iter()
            .map(|line| VatBreakdownLineJson {
                rate_bp: line.rate_bp,
                vat_cents: line.vat_cents,
            })
            .collect::<Vec<_>>(),
    )
    .expect("a vec of plain structs always serializes")
}

impl TryFrom<QuoteLineRow> for QuoteLine {
    type Error = CoreError;

    fn try_from(row: QuoteLineRow) -> Result<Self, Self::Error> {
        let discount_percent = DiscountPercent::try_from(row.discount_percent).map_err(|e| {
            CoreError::Internal(format!("invalid quote line discount in database: {e}"))
        })?;
        let billing_frequency = BillingFrequency::from_str(&row.billing_frequency)
            .map_err(|e| CoreError::Internal(format!("invalid quote line frequency: {e}")))?;

        Ok(Self {
            id: QuoteLineId(row.id),
            organization_id: OrganizationId(row.org_id),
            quote_id: QuoteId(row.quote_id),
            service_rate_id: row.service_rate_id.map(ServiceRateId),
            product_id: row.product_id.map(ProductId),
            label: row.label,
            quantity: row.quantity,
            unit: row.unit,
            unit_price_cents: row.unit_price_cents,
            vat_rate_bp: row.vat_rate_bp,
            discount_percent,
            billing_frequency,
            notes: row.notes,
            photo_keys: row.photo_keys,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub fn snapshot_to_json<T: serde::Serialize>(snapshot: Option<&T>) -> Option<serde_json::Value> {
    snapshot.map(|value| {
        serde_json::to_value(value).expect("a snapshot of plain fields always serializes")
    })
}
