use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, Quote, QuoteId, QuoteStatus};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait QuoteRepository: Send {
    /// Atomically bumps the organization's counter for `year` and returns
    /// the number it lands on, formatted `{prefix}-{year}-{counter:04}`.
    /// Safe under contention: see the migration that creates
    /// `quote_number_counters` for why.
    fn allocate_number(
        &mut self,
        organization_id: OrganizationId,
        prefix: &str,
        year: i32,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;

    fn insert(&mut self, quote: &Quote) -> impl Future<Output = Result<Quote, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: QuoteId,
    ) -> impl Future<Output = Result<Option<Quote>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Quote>, u64), CoreError>> + Send;

    fn update(&mut self, quote: &Quote) -> impl Future<Output = Result<Quote, CoreError>> + Send;

    /// `reference` is `Some` only on the call that allocates it — the
    /// transition into `Sent` that finds none yet. Every other call passes
    /// `None`, which leaves whatever the row already holds untouched.
    fn update_status(
        &mut self,
        id: QuoteId,
        status: QuoteStatus,
        reference: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Quote, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: QuoteId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
