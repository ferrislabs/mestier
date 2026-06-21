use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, Product, ProductId};

#[cfg_attr(test, mockall::automock)]
pub trait ProductRepository: Send {
    fn insert(
        &mut self,
        product: &Product,
    ) -> impl Future<Output = Result<Product, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ProductId,
    ) -> impl Future<Output = Result<Option<Product>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Product>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        product: &Product,
    ) -> impl Future<Output = Result<Product, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: ProductId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
