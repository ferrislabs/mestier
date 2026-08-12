use common::CoreError;
use discord::{
    Category, CategoryId, CategoryRepository, CategoryService, CreateCategoryCommand,
    OrganizationId, UpdateCategoryCommand,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(category, events)]
    pub async fn create_category(&self, cmd: CreateCategoryCommand) -> Result<Category, CoreError> {
        let mut service = CategoryService::new(category_repository, &events);
        let result = service.create_category(cmd).await?;
        Ok(result)
    }

    #[transactional(category, events)]
    pub async fn update_category(&self, cmd: UpdateCategoryCommand) -> Result<Category, CoreError> {
        let mut service = CategoryService::new(category_repository, &events);
        let result = service.update_category(cmd).await?;
        Ok(result)
    }

    #[transactional(category, events)]
    pub async fn delete_category(&self, id: CategoryId) -> Result<(), CoreError> {
        let mut service = CategoryService::new(category_repository, &events);
        service.delete_category(id).await?;
        Ok(())
    }

    #[transactional(category)]
    pub async fn get_category(&self, id: CategoryId) -> Result<Category, CoreError> {
        let mut repo = category_repository;
        repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    #[transactional(category, events)]
    pub async fn list_categories(&self, org: OrganizationId) -> Result<Vec<Category>, CoreError> {
        let mut service = CategoryService::new(category_repository, &events);
        service.list_categories(org).await
    }
}
