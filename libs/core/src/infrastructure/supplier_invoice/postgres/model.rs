use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    OrganizationId, SupplierId, SupplierInvoice, SupplierInvoiceId, SupplierInvoiceLine,
    SupplierInvoiceLineId, SupplierInvoiceSource, SupplierInvoiceStatus,
    SupplierInvoiceVatBreakdownLine,
};

#[derive(Debug, Clone)]
pub struct SupplierInvoiceRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: String,
    pub supplier_registration_number: Option<String>,
    pub supplier_vat_number: Option<String>,
    pub number: String,
    pub issued_on: NaiveDate,
    pub due_on: Option<NaiveDate>,
    pub received_at: DateTime<Utc>,
    pub source: String,
    pub status: String,
    pub currency: String,
    pub source_file_key: Option<String>,
    pub source_file_mime_type: Option<String>,
    pub notes: Option<String>,
    pub net_cents: i32,
    pub vat_breakdown: serde_json::Value,
    pub gross_cents: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SupplierInvoiceRow {
    pub fn into_supplier_invoice(
        self,
        lines: Vec<SupplierInvoiceLine>,
    ) -> Result<SupplierInvoice, CoreError> {
        let status = SupplierInvoiceStatus::from_str(&self.status).map_err(|e| {
            CoreError::Internal(format!("invalid supplier invoice status in database: {e}"))
        })?;
        let source = SupplierInvoiceSource::from_str(&self.source).map_err(|e| {
            CoreError::Internal(format!("invalid supplier invoice source in database: {e}"))
        })?;
        let vat_breakdown = parse_vat_breakdown(self.vat_breakdown)?;

        Ok(SupplierInvoice {
            id: SupplierInvoiceId(self.id),
            organization_id: OrganizationId(self.org_id),
            supplier_id: self.supplier_id.map(SupplierId),
            supplier_name: self.supplier_name,
            supplier_registration_number: self.supplier_registration_number,
            supplier_vat_number: self.supplier_vat_number,
            number: self.number,
            issued_on: self.issued_on,
            due_on: self.due_on,
            received_at: self.received_at,
            source,
            status,
            currency: self.currency,
            source_file_key: self.source_file_key,
            source_file_mime_type: self.source_file_mime_type,
            notes: self.notes,
            net_cents: self.net_cents,
            vat_breakdown,
            gross_cents: self.gross_cents,
            lines,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SupplierInvoiceLineRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub supplier_invoice_id: Uuid,
    pub label: String,
    pub quantity: Decimal,
    pub unit: Option<String>,
    pub unit_price_cents: i32,
    pub line_total_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
    pub position: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<SupplierInvoiceLineRow> for SupplierInvoiceLine {
    type Error = CoreError;

    fn try_from(row: SupplierInvoiceLineRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: SupplierInvoiceLineId(row.id),
            organization_id: OrganizationId(row.org_id),
            supplier_invoice_id: SupplierInvoiceId(row.supplier_invoice_id),
            label: row.label,
            quantity: row.quantity,
            unit: row.unit,
            unit_price_cents: row.unit_price_cents,
            line_total_cents: row.line_total_cents,
            vat_rate_basis_points: row.vat_rate_basis_points,
            position: row.position,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// The JSONB column round-trips through this shape rather than deriving
/// `Serialize`/`Deserialize` on the domain type directly — same device as
/// `invoice::postgres::model::VatBreakdownLineJson`.
#[derive(serde::Serialize, serde::Deserialize)]
struct VatBreakdownLineJson {
    rate_bp: i32,
    vat_cents: i32,
}

fn parse_vat_breakdown(
    value: serde_json::Value,
) -> Result<Vec<SupplierInvoiceVatBreakdownLine>, CoreError> {
    let lines: Vec<VatBreakdownLineJson> = serde_json::from_value(value).map_err(|e| {
        CoreError::Internal(format!(
            "invalid supplier invoice vat_breakdown in database: {e}"
        ))
    })?;

    Ok(lines
        .into_iter()
        .map(|line| SupplierInvoiceVatBreakdownLine {
            rate_bp: line.rate_bp,
            vat_cents: line.vat_cents,
        })
        .collect())
}

pub fn vat_breakdown_to_json(breakdown: &[SupplierInvoiceVatBreakdownLine]) -> serde_json::Value {
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
