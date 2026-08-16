use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{CustomerContext, CustomerContextId, CustomerId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CustomerContextRepository: Send {
    fn insert(
        &mut self,
        customer_context: &CustomerContext,
    ) -> impl Future<Output = Result<CustomerContext, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: CustomerContextId,
    ) -> impl Future<Output = Result<Option<CustomerContext>, CoreError>> + Send;

    fn list_by_customer(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<CustomerContext>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        customer_context: &CustomerContext,
    ) -> impl Future<Output = Result<CustomerContext, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: CustomerContextId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
