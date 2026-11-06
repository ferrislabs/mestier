use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{CustomerId, OrganizationId, QuoteId, QuoteStatus};

pub const MAX_QUOTE_SEARCH_RESULTS: u64 = 200;
pub const DEFAULT_QUOTE_SEARCH_RESULTS: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("a quote search returns between 1 and {MAX_QUOTE_SEARCH_RESULTS} quotes")]
pub struct QuoteSearchLimitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuoteSearchLimit(u16);

impl QuoteSearchLimit {
    pub fn new(requested: u64) -> Result<Self, QuoteSearchLimitError> {
        if (1..=MAX_QUOTE_SEARCH_RESULTS).contains(&requested) {
            u16::try_from(requested)
                .map(Self)
                .map_err(|_| QuoteSearchLimitError)
        } else {
            Err(QuoteSearchLimitError)
        }
    }

    pub fn get(self) -> usize {
        usize::from(self.0)
    }

    pub fn as_i64(self) -> i64 {
        i64::from(self.0)
    }
}

impl Default for QuoteSearchLimit {
    fn default() -> Self {
        Self(DEFAULT_QUOTE_SEARCH_RESULTS as u16)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteSearch {
    pub organization_id: OrganizationId,
    pub status: Option<QuoteStatus>,
    pub updated_before: Option<DateTime<Utc>>,
    pub limit: QuoteSearchLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteSummary {
    pub id: QuoteId,
    pub reference: Option<String>,
    pub title: String,
    pub customer_id: CustomerId,
    pub status: QuoteStatus,
    pub net_cents: i32,
    pub gross_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteSearchResults {
    items: Vec<QuoteSummary>,
}

impl QuoteSearchResults {
    pub fn bounded(mut items: Vec<QuoteSummary>, limit: QuoteSearchLimit) -> Self {
        items.truncate(limit.get());
        Self { items }
    }

    pub fn items(&self) -> &[QuoteSummary] {
        &self.items
    }

    pub fn into_items(self) -> Vec<QuoteSummary> {
        self.items
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn summary(n: u128) -> QuoteSummary {
        let at = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap();
        QuoteSummary {
            id: QuoteId(Uuid::from_u128(n)),
            reference: None,
            title: format!("Quote {n}"),
            customer_id: CustomerId(Uuid::from_u128(1)),
            status: QuoteStatus::Sent,
            net_cents: 100,
            gross_cents: 120,
            created_at: at,
            updated_at: at,
        }
    }

    #[test]
    fn a_limit_is_between_one_and_two_hundred() {
        assert!(QuoteSearchLimit::new(1).is_ok());
        assert_eq!(QuoteSearchLimit::new(200).unwrap().get(), 200);
        for refused in [0, 201, 10_000, u64::MAX] {
            assert_eq!(QuoteSearchLimit::new(refused), Err(QuoteSearchLimitError));
        }
    }

    #[test]
    fn the_default_limit_is_fifty() {
        assert_eq!(QuoteSearchLimit::default().get(), 50);
    }

    #[test]
    fn results_never_hold_more_than_the_limit() {
        let many: Vec<QuoteSummary> = (0..300).map(summary).collect();

        let results =
            QuoteSearchResults::bounded(many.clone(), QuoteSearchLimit::new(200).unwrap());
        let fewer = QuoteSearchResults::bounded(many, QuoteSearchLimit::new(3).unwrap());

        assert_eq!(results.items().len(), 200);
        assert_eq!(fewer.into_items().len(), 3);
    }

    #[test]
    fn results_below_the_limit_are_kept_whole() {
        let few: Vec<QuoteSummary> = (0..2).map(summary).collect();

        let results = QuoteSearchResults::bounded(few.clone(), QuoteSearchLimit::default());

        assert_eq!(results.into_items(), few);
    }
}
