use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{OrganizationId, Product, ProductId};

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
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
