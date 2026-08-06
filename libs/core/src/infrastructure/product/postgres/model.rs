use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{OrganizationId, Product, ProductId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct ProductRow {
	pub id: Uuid,
	pub org_id: Uuid,
	pub name: String,
	pub sku: Option<String>,
	pub unit: String,
	pub unit_price_cents: i32,
	pub vat_rate: Decimal,
	pub custom_fields: Json<Value>,
	pub description: Option<String>,
	pub deleted_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl TryFrom<ProductRow> for Product {
	type Error = CoreError;

	fn try_from(row: ProductRow) -> Result<Self, Self::Error> {
		let unit = ServiceRateUnit::from_str(&row.unit)
			.map_err(|e| CoreError::Internal(format!("invalid product unit in database: {e}")))?;

		let custom_fields: HashMap<String, String> = serde_json::from_value(row.custom_fields.0)
			.map_err(|e| {
				CoreError::Internal(format!("invalid product custom_fields in database: {e}"))
			})?;

		Ok(Self {
			id: ProductId(row.id),
			organization_id: OrganizationId(row.org_id),
			name: row.name,
			sku: row.sku,
			unit,
			unit_price_cents: row.unit_price_cents,
			vat_rate: row.vat_rate,
			custom_fields,
			description: row.description,
			deleted_at: row.deleted_at,
			created_at: row.created_at,
			updated_at: row.updated_at,
		})
	}
}
