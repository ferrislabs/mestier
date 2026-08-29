use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, UserId};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    CustomerContextId, CustomerId, CustomerOutstandingBalance, GeneratedInvoiceDocument, Invoice,
    InvoiceId, InvoiceKind, InvoiceLine, InvoiceLineId, InvoicePayment, InvoicePaymentId,
    InvoiceStatus, InvoiceVatBreakdownLine, LegalIdentity, OperationNature, OrganizationAddress,
    OrganizationId, ProjectId, VatStatus,
};

#[derive(Debug, Clone)]
pub struct InvoiceRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub number: Option<String>,
    pub kind: String,
    pub project_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub customer_context_id: Uuid,
    pub status: String,
    pub issued_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub operation_nature: Option<String>,
    pub delivery_address_line1: Option<String>,
    pub delivery_address_line2: Option<String>,
    pub delivery_address_postal_code: Option<String>,
    pub delivery_address_city: Option<String>,
    pub delivery_address_country: Option<String>,
    pub net_cents: i32,
    pub vat_breakdown: serde_json::Value,
    pub gross_cents: i32,
    pub issuer_identity: Option<serde_json::Value>,
    pub document_format: Option<String>,
    pub document_file_key: Option<String>,
    pub document_mime_type: Option<String>,
    pub document_generated_at: Option<DateTime<Utc>>,
    pub source_invoice_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct InvoiceLineRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub invoice_id: Uuid,
    pub label: String,
    pub quantity: Decimal,
    pub unit_price_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
    pub position: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvoiceRow {
    pub fn into_invoice(self, lines: Vec<InvoiceLine>) -> Result<Invoice, CoreError> {
        let status = InvoiceStatus::from_str(&self.status)
            .map_err(|e| CoreError::Internal(format!("invalid invoice status in database: {e}")))?;
        let kind = InvoiceKind::from_str(&self.kind)
            .map_err(|e| CoreError::Internal(format!("invalid invoice kind in database: {e}")))?;
        let vat_breakdown = parse_vat_breakdown(self.vat_breakdown)?;
        let issuer_identity = self.issuer_identity.map(parse_legal_identity).transpose()?;
        let operation_nature = self
            .operation_nature
            .map(|value| OperationNature::from_str(&value))
            .transpose()
            .map_err(|e| {
                CoreError::Internal(format!("invalid invoice operation_nature in database: {e}"))
            })?;
        let delivery_address = self
            .delivery_address_line1
            .map(|line1| OrganizationAddress {
                line1,
                line2: self.delivery_address_line2,
                postal_code: self.delivery_address_postal_code.unwrap_or_default(),
                city: self.delivery_address_city.unwrap_or_default(),
                country: self.delivery_address_country.unwrap_or_default(),
            });
        let generated_document = self.document_format.map(|format| GeneratedInvoiceDocument {
            format,
            file_key: self.document_file_key.unwrap_or_default(),
            mime_type: self.document_mime_type.unwrap_or_default(),
            generated_at: self.document_generated_at.unwrap_or(self.updated_at),
        });

        Ok(Invoice {
            id: InvoiceId(self.id),
            organization_id: OrganizationId(self.org_id),
            number: self.number,
            kind,
            project_id: self.project_id.map(ProjectId),
            customer_id: CustomerId(self.customer_id),
            customer_context_id: CustomerContextId(self.customer_context_id),
            status,
            issued_at: self.issued_at,
            due_at: self.due_at,
            notes: self.notes,
            operation_nature,
            delivery_address,
            net_cents: self.net_cents,
            vat_breakdown,
            gross_cents: self.gross_cents,
            issuer_identity,
            generated_document,
            lines,
            source_invoice_id: self.source_invoice_id.map(InvoiceId),
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// The JSONB column round-trips through this shape rather than deriving
/// `Serialize`/`Deserialize` on the domain type directly — same device as
/// `quote::postgres::model::vat_breakdown_to_json`.
#[derive(serde::Serialize, serde::Deserialize)]
struct VatBreakdownLineJson {
    rate_bp: i32,
    vat_cents: i32,
}

fn parse_vat_breakdown(
    value: serde_json::Value,
) -> Result<Vec<InvoiceVatBreakdownLine>, CoreError> {
    let lines: Vec<VatBreakdownLineJson> = serde_json::from_value(value).map_err(|e| {
        CoreError::Internal(format!("invalid invoice vat_breakdown in database: {e}"))
    })?;

    Ok(lines
        .into_iter()
        .map(|line| InvoiceVatBreakdownLine {
            rate_bp: line.rate_bp,
            vat_cents: line.vat_cents,
        })
        .collect())
}

pub fn vat_breakdown_to_json(breakdown: &[InvoiceVatBreakdownLine]) -> serde_json::Value {
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

/// The five `delivery_address_*` columns, in the order every query below
/// binds them: line1, line2, postal code, city, country.
type DeliveryAddressColumns<'a> = (
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
);

/// The five `delivery_address_*` columns to bind, or `NULL` all around when
/// the invoice carries none — mirrors `vat_status_columns` on the
/// organization repository: one function decides how an `Option<Struct>`
/// spreads across several nullable columns, used by every query that
/// writes it.
pub fn delivery_address_columns(
    address: &Option<OrganizationAddress>,
) -> DeliveryAddressColumns<'_> {
    match address {
        Some(address) => (
            Some(address.line1.as_str()),
            address.line2.as_deref(),
            Some(address.postal_code.as_str()),
            Some(address.city.as_str()),
            Some(address.country.as_str()),
        ),
        None => (None, None, None, None, None),
    }
}

/// The JSON mirror of `LegalIdentity`, kept private to this module for the
/// same reason `VatBreakdownLineJson` is: the domain type stays free of any
/// hint that it is ever persisted, and this struct is the one place that
/// decides the wire shape of `issuer_identity`.
#[derive(serde::Serialize, serde::Deserialize)]
struct LegalIdentityJson {
    legal_name: String,
    legal_form: String,
    registration_number: String,
    vat_status: VatStatusJson,
    share_capital_cents: Option<i64>,
    address_line1: String,
    address_line2: Option<String>,
    address_postal_code: String,
    address_city: String,
    address_country: String,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    insurance_mention: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum VatStatusJson {
    Subject { vat_number: String },
    NotSubject { basis: String },
}

pub fn legal_identity_to_json(identity: &LegalIdentity) -> serde_json::Value {
    let vat_status = match &identity.vat_status {
        VatStatus::Subject { vat_number } => VatStatusJson::Subject {
            vat_number: vat_number.clone(),
        },
        VatStatus::NotSubject { basis } => VatStatusJson::NotSubject {
            basis: basis.clone(),
        },
    };

    serde_json::to_value(LegalIdentityJson {
        legal_name: identity.legal_name.clone(),
        legal_form: identity.legal_form.clone(),
        registration_number: identity.registration_number.clone(),
        vat_status,
        share_capital_cents: identity.share_capital_cents,
        address_line1: identity.address.line1.clone(),
        address_line2: identity.address.line2.clone(),
        address_postal_code: identity.address.postal_code.clone(),
        address_city: identity.address.city.clone(),
        address_country: identity.address.country.clone(),
        contact_email: identity.contact_email.clone(),
        contact_phone: identity.contact_phone.clone(),
        insurance_mention: identity.insurance_mention.clone(),
    })
    .expect("a plain struct of owned fields always serializes")
}

fn parse_legal_identity(value: serde_json::Value) -> Result<LegalIdentity, CoreError> {
    let parsed: LegalIdentityJson = serde_json::from_value(value).map_err(|e| {
        CoreError::Internal(format!("invalid invoice issuer_identity in database: {e}"))
    })?;

    let vat_status = match parsed.vat_status {
        VatStatusJson::Subject { vat_number } => VatStatus::Subject { vat_number },
        VatStatusJson::NotSubject { basis } => VatStatus::NotSubject { basis },
    };

    Ok(LegalIdentity {
        legal_name: parsed.legal_name,
        legal_form: parsed.legal_form,
        registration_number: parsed.registration_number,
        vat_status,
        share_capital_cents: parsed.share_capital_cents,
        address: OrganizationAddress {
            line1: parsed.address_line1,
            line2: parsed.address_line2,
            postal_code: parsed.address_postal_code,
            city: parsed.address_city,
            country: parsed.address_country,
        },
        contact_email: parsed.contact_email,
        contact_phone: parsed.contact_phone,
        insurance_mention: parsed.insurance_mention,
    })
}

#[derive(Debug, Clone)]
pub struct InvoicePaymentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub invoice_id: Uuid,
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub recorded_by: Uuid,
    pub deleted_by: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<InvoicePaymentRow> for InvoicePayment {
    type Error = CoreError;

    fn try_from(row: InvoicePaymentRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: InvoicePaymentId(row.id),
            organization_id: OrganizationId(row.org_id),
            invoice_id: InvoiceId(row.invoice_id),
            amount_cents: row.amount_cents,
            paid_on: row.paid_on,
            method: row.method,
            reference: row.reference,
            note: row.note,
            recorded_by: UserId(row.recorded_by),
            deleted_at: row.deleted_at,
            deleted_by: row.deleted_by.map(UserId),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// The result shape of `list_outstanding_by_customer`'s aggregate query —
/// `sqlx::query_as!` needs a plain struct of its own rather than
/// `InvoiceRow`, since this is not a row of `invoices` but one summary per
/// customer.
#[derive(Debug, Clone)]
pub struct CustomerOutstandingBalanceRow {
    pub customer_id: Uuid,
    pub outstanding_cents: i64,
    pub oldest_due_at: Option<DateTime<Utc>>,
}

impl From<CustomerOutstandingBalanceRow> for CustomerOutstandingBalance {
    fn from(row: CustomerOutstandingBalanceRow) -> Self {
        Self {
            customer_id: CustomerId(row.customer_id),
            outstanding_cents: row.outstanding_cents,
            oldest_due_at: row.oldest_due_at,
        }
    }
}

impl TryFrom<InvoiceLineRow> for InvoiceLine {
    type Error = CoreError;

    fn try_from(row: InvoiceLineRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: InvoiceLineId(row.id),
            organization_id: OrganizationId(row.org_id),
            invoice_id: InvoiceId(row.invoice_id),
            label: row.label,
            quantity: row.quantity,
            unit_price_cents: row.unit_price_cents,
            vat_rate_basis_points: row.vat_rate_basis_points,
            position: row.position,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
