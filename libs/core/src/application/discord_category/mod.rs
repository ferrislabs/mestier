use common::CoreError;
use discord::{
    Category, CategoryId, CategoryRepository, CategoryService, CreateCategoryCommand,
    OrganizationId, UpdateCategoryCommand,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(category)]
    pub async fn create_category(&self, cmd: CreateCategoryCommand) -> Result<Category, CoreError> {
        let org_id = cmd.organization_id;
        let mut service = CategoryService::new(category_repository, self.events.as_ref());
        let result = service.create_category(cmd).await?;
        // best-effort flush at end of tx closure; events are reconciled via REST (spec §2)
        self.events.flush(org_id);
        Ok(result)
    }

    #[transactional(category)]
    pub async fn update_category(&self, cmd: UpdateCategoryCommand) -> Result<Category, CoreError> {
        let mut service = CategoryService::new(category_repository, self.events.as_ref());
        let result = service.update_category(cmd).await?;
        self.events.flush(result.organization_id);
        Ok(result)
    }

    #[transactional(category)]
    pub async fn delete_category(&self, id: CategoryId) -> Result<(), CoreError> {
        // Load org_id before deleting so we can flush events with the correct org.
        let mut repo = category_repository;
        let existing = repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        let org_id = existing.organization_id;
        let mut service = CategoryService::new(repo, self.events.as_ref());
        service.delete_category(id).await?;
        self.events.flush(org_id);
        Ok(())
    }

    #[transactional(category)]
    pub async fn get_category(&self, id: CategoryId) -> Result<Category, CoreError> {
        let mut repo = category_repository;
        repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    #[transactional(category)]
    pub async fn list_categories(&self, org: OrganizationId) -> Result<Vec<Category>, CoreError> {
        let mut service = CategoryService::new(category_repository, self.events.as_ref());
        service.list_categories(org).await
    }
}
