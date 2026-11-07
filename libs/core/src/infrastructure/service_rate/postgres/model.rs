use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    BillingFrequency, CatalogProfile, CatalogUsage, OrganizationId, ServiceRate, ServiceRateId,
};

#[derive(Debug, Clone)]
pub struct ServiceRateRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub label: String,
    pub unit: String,
    pub rate_cents: i32,
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
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

impl TryFrom<ServiceRateRow> for ServiceRate {
    type Error = CoreError;

    fn try_from(row: ServiceRateRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ServiceRateId(row.id),
            organization_id: OrganizationId(row.org_id),
            label: row.label,
            unit: row.unit,
            rate_cents: row.rate_cents,
            default_vat_rate_bp: row.default_vat_rate_bp,
            description: row.description,
            catalog: CatalogProfile {
                category: row.category,
                billing_frequency: BillingFrequency::from_str(&row.billing_frequency).map_err(
                    |error| {
                        CoreError::Internal(format!(
                            "invalid service rate billing frequency in database: {error}"
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
