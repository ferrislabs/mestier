use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    BillingFrequency, CatalogProfile, CatalogUsage, OrganizationId, Product, ProductId, SupplierId,
};

#[derive(Debug, Clone)]
pub struct ProductRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub unit: String,
    pub unit_price_cents: i32,
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
    pub photo_keys: Vec<String>,
    pub supplier_id: Option<Uuid>,
    pub manufacturer: Option<String>,
    pub technical_specs: Option<String>,
    pub datasheet_keys: Vec<String>,
    pub category: String,
    pub billing_frequency: String,
    pub cost_cents: Option<i32>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub quote_usage_count: i64,
    pub invoice_usage_count: i64,
}

impl TryFrom<ProductRow> for Product {
    type Error = CoreError;

    fn try_from(row: ProductRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ProductId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            sku: row.sku,
            unit: row.unit,
            unit_price_cents: row.unit_price_cents,
            default_vat_rate_bp: row.default_vat_rate_bp,
            description: row.description,
            photo_keys: row.photo_keys,
            supplier_id: row.supplier_id.map(SupplierId),
            manufacturer: row.manufacturer,
            technical_specs: row.technical_specs,
            datasheet_keys: row.datasheet_keys,
            catalog: CatalogProfile {
                category: row.category,
                billing_frequency: BillingFrequency::from_str(&row.billing_frequency).map_err(
                    |error| {
                        CoreError::Internal(format!(
                            "invalid product billing frequency in database: {error}"
                        ))
                    },
                )?,
                cost_cents: row.cost_cents,
                archived_at: row.archived_at,
            },
            usage: CatalogUsage {
                quote_lines: row.quote_usage_count,
                invoice_lines: row.invoice_usage_count,
            },
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
