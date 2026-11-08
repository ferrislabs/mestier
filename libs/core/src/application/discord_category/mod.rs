use common::CoreError;
use discord::{
    Category, CategoryAccess, CategoryAccessRepository, CategoryId, CategoryRepository,
    CategoryService, CreateCategoryCommand, OrganizationId, RoleId, UpdateCategoryCommand, UserId,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;
use crate::domain::{member::ports::MemberRepository, role::ports::RoleRepository};

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

    /// Creates a category only `role_ids` (and its creator) can see into.
    ///
    /// No event is published: the gateway sends category events to the whole
    /// organization, and a private category's name must not leave with them.
    /// The creator is listed as a member of their own, so they cannot lock
    /// themselves out.
    #[transactional(category, category_access, role, events)]
    pub async fn create_private_category(
        &self,
        cmd: CreateCategoryCommand,
        creator: UserId,
        role_ids: Vec<RoleId>,
    ) -> Result<Category, CoreError> {
        if role_ids.is_empty() {
            return Err(CoreError::Conflict(
                "a private category needs at least one role".to_owned(),
            ));
        }
        let organization_id = cmd.organization_id;
        let mut role_repository = role_repository;
        let org_roles = role_repository
            .list_by_organization(organization_id)
            .await?;
        if let Some(unknown) = role_ids
            .iter()
            .find(|role_id| !org_roles.iter().any(|role| role.id == **role_id))
        {
            return Err(CoreError::Conflict(format!(
                "role {} does not belong to this organization",
                unknown.0
            )));
        }

        let mut service = CategoryService::new(category_repository, &events);
        let category = service.create_category_unannounced(cmd).await?;
        let mut category_access_repository = category_access_repository;
        category_access_repository
            .replace(
                organization_id,
                category.id,
                &CategoryAccess {
                    roles: role_ids,
                    users: vec![creator],
                },
            )
            .await?;
        Ok(category)
    }

    /// The categories `user` may see: the open ones, and the private ones whose
    /// access list admits them.
    #[transactional(category, category_access, member)]
    pub async fn list_visible_categories(
        &self,
        user: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<Category>, CoreError> {
        let mut member_repository = member_repository;
        let member = member_repository
            .find_by_org_and_user(organization_id, user)
            .await?
            .ok_or(CoreError::Forbidden {
                reason: Some("not a member of this organization".to_owned()),
            })?;
        let role_ids = member_repository.list_role_ids(member.id).await?;

        let mut category_repository = category_repository;
        let categories = category_repository
            .list_by_organization(organization_id)
            .await?;
        let mut category_access_repository = category_access_repository;
        let private = category_access_repository
            .list_for_organization(organization_id)
            .await?;
        Ok(categories
            .into_iter()
            .filter(|category| {
                private
                    .get(&category.id)
                    .is_none_or(|access| access.admits(user, &role_ids))
            })
            .collect())
    }
}
