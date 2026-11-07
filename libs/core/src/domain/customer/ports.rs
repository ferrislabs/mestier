use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{Customer, CustomerContactId, CustomerContextId, CustomerId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CustomerRepository: Send {
    fn insert(
        &mut self,
        customer: &Customer,
    ) -> impl Future<Output = Result<Customer, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: CustomerId,
    ) -> impl Future<Output = Result<Option<Customer>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Customer>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        customer: &Customer,
    ) -> impl Future<Output = Result<Customer, CoreError>> + Send;

    fn has_contact(
        &mut self,
        customer_id: CustomerId,
        contact_id: CustomerContactId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn has_address(
        &mut self,
        customer_id: CustomerId,
        address_id: CustomerContextId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: CustomerId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
