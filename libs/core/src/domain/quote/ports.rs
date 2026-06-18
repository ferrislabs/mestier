use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, Quote, QuoteId, QuoteStatus};

#[cfg_attr(test, mockall::automock)]
pub trait QuoteRepository: Send {
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

    fn update_status(
        &mut self,
        id: QuoteId,
        status: QuoteStatus,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Quote, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: QuoteId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
