use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{OrganizationId, ServiceRate, ServiceRateId, ServiceRateUnit};

#[derive(Debug, Clone)]
pub struct ServiceRateRow {
	pub id: Uuid,
	pub org_id: Uuid,
	pub label: String,
	pub unit: String,
	pub rate_cents: i32,
	pub vat_rate: Decimal,
	pub custom_fields: Json<Value>,
	pub deleted_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl TryFrom<ServiceRateRow> for ServiceRate {
	type Error = CoreError;

	fn try_from(row: ServiceRateRow) -> Result<Self, Self::Error> {
		let unit = ServiceRateUnit::from_str(&row.unit).map_err(|e| {
			CoreError::Internal(format!("invalid service rate unit in database: {e}"))
		})?;

		let custom_fields: HashMap<String, String> = serde_json::from_value(row.custom_fields.0)
			.map_err(|e| {
				CoreError::Internal(format!(
					"invalid service rate custom_fields in database: {e}"
				))
			})?;

		Ok(Self {
			id: ServiceRateId(row.id),
			organization_id: OrganizationId(row.org_id),
			label: row.label,
			unit,
			rate_cents: row.rate_cents,
			vat_rate: row.vat_rate,
			custom_fields,
			deleted_at: row.deleted_at,
			created_at: row.created_at,
			updated_at: row.updated_at,
		})
	}
}
