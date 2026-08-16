use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{CustomerContact, CustomerContactId, CustomerId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CustomerContactRepository: Send {
    fn insert(
        &mut self,
        customer_contact: &CustomerContact,
    ) -> impl Future<Output = Result<CustomerContact, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: CustomerContactId,
    ) -> impl Future<Output = Result<Option<CustomerContact>, CoreError>> + Send;

    fn list_by_customer(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<CustomerContact>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        customer_contact: &CustomerContact,
    ) -> impl Future<Output = Result<CustomerContact, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: CustomerContactId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
