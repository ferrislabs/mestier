use chrono::{DateTime, Utc};
use common::CoreError;

use crate::BillingFrequency;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CatalogProfile {
    pub category: String,
    pub billing_frequency: BillingFrequency,
    pub cost_cents: Option<i32>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CatalogUsage {
    pub quote_lines: i64,
    pub invoice_lines: i64,
}

impl CatalogUsage {
    pub fn total(self) -> i64 {
        self.quote_lines + self.invoice_lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CatalogProfileInput {
    pub category: String,
    pub billing_frequency: BillingFrequency,
    pub cost_cents: Option<i32>,
    pub archived: bool,
}

impl CatalogProfileInput {
    pub fn resolve(
        input: Option<Self>,
        current: Option<&CatalogProfile>,
        now: DateTime<Utc>,
    ) -> Result<CatalogProfile, CoreError> {
        let Some(input) = input else {
            return Ok(current.cloned().unwrap_or_default());
        };
        if input.cost_cents.is_some_and(|cost| cost < 0) {
            return Err(CoreError::Conflict("cost cannot be negative".to_owned()));
        }
        let already_archived = current.and_then(|profile| profile.archived_at);
        Ok(CatalogProfile {
            category: input.category.trim().to_owned(),
            billing_frequency: input.billing_frequency,
            cost_cents: input.cost_cents,
            archived_at: input.archived.then(|| already_archived.unwrap_or(now)),
        })
    }
}
