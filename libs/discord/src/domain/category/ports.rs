use common::CoreError;

use crate::{Category, CategoryId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CategoryRepository: Send {
    fn insert(&mut self, c: &Category) -> impl Future<Output = Result<Category, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: CategoryId,
    ) -> impl Future<Output = Result<Option<Category>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        org: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Category>, CoreError>> + Send;

    fn update(&mut self, c: &Category) -> impl Future<Output = Result<Category, CoreError>> + Send;

    fn delete(&mut self, id: CategoryId) -> impl Future<Output = Result<(), CoreError>> + Send;
}
