use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{CustomerId, Property, PropertyId};

#[cfg_attr(test, mockall::automock)]
pub trait PropertyRepository: Send {
    fn insert(
        &mut self,
        property: &Property,
    ) -> impl Future<Output = Result<Property, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: PropertyId,
    ) -> impl Future<Output = Result<Option<Property>, CoreError>> + Send;

    fn list_by_customer(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Property>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        property: &Property,
    ) -> impl Future<Output = Result<Property, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: PropertyId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
