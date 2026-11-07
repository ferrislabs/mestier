use chrono::{DateTime, Utc};
use common::CoreError;

use super::search::{QuoteSearch, QuoteSummary};
use crate::{
    DealId, OrganizationId, Quote, QuoteId, domain::organization::billing_settings::NumberPadding,
};

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
        padding: NumberPadding,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;

    fn last_assigned_number(
        &mut self,
        organization_id: OrganizationId,
        year: i32,
    ) -> impl Future<Output = Result<i32, CoreError>> + Send;

    fn raise_last_assigned_number(
        &mut self,
        organization_id: OrganizationId,
        year: i32,
        last_assigned: i32,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn insert(&mut self, quote: &Quote) -> impl Future<Output = Result<Quote, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: QuoteId,
    ) -> impl Future<Output = Result<Option<Quote>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        deal_id: Option<DealId>,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Quote>, u64), CoreError>> + Send;

    fn search(
        &mut self,
        search: &QuoteSearch,
    ) -> impl Future<Output = Result<Vec<QuoteSummary>, CoreError>> + Send;

    fn update(&mut self, quote: &Quote) -> impl Future<Output = Result<Quote, CoreError>> + Send;

    fn update_lifecycle(
        &mut self,
        quote: &Quote,
    ) -> impl Future<Output = Result<Quote, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: QuoteId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
