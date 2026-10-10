use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    DoeDocument, DoeModel, DoeModelId, DoeSettings, OrganizationId,
    application::{MestierUseCase, policy},
    domain::doe_model::{
        commands::{
            CreateDoeModelCommand, CreateDoeModelFromStarterCommand, PreviewDoeStyleCommand,
            UpdateDoeModelCommand, UpdateDoeSettingsCommand,
        },
        service::DoeModelService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(doe_model, doe_settings, role, member, authz)]
    pub async fn create_doe_model(
        &self,
        command: CreateDoeModelCommand,
    ) -> Result<DoeModel, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        service.create_model(command).await
    }

    #[transactional(doe_model, doe_settings)]
    pub async fn get_doe_model(&self, id: DoeModelId) -> Result<DoeModel, CoreError> {
        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        service.get_model(id).await
    }

    #[transactional(doe_model, doe_settings)]
    pub async fn list_doe_models(
        &self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<DoeModel>, u64), CoreError> {
        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        service
            .list_models(organization_id, include_archived, limit, offset)
            .await
    }

    #[transactional(doe_model, doe_settings, role, member, authz)]
    pub async fn update_doe_model(
        &self,
        command: UpdateDoeModelCommand,
    ) -> Result<DoeModel, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        let existing = service.get_model(command.id).await?;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        service.update_model(command).await
    }

    #[transactional(doe_model, doe_settings, role, member, authz)]
    pub async fn archive_doe_model(
        &self,
        actor: authz::Subject,
        id: DoeModelId,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        let existing = service.get_model(id).await?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        service.archive_model(id).await
    }

    #[transactional(doe_model, doe_settings, role, member, authz)]
    pub async fn restore_doe_model(
        &self,
        actor: authz::Subject,
        id: DoeModelId,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        let existing = service.get_model(id).await?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        service.restore_model(id).await
    }

    #[transactional(doe_model, doe_settings, role, member, authz)]
    pub async fn create_doe_model_from_starter(
        &self,
        command: CreateDoeModelFromStarterCommand,
    ) -> Result<DoeModel, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        service.create_from_starter(command).await
    }

    #[transactional(doe_model, doe_settings)]
    pub async fn get_doe_settings(
        &self,
        organization_id: OrganizationId,
    ) -> Result<DoeSettings, CoreError> {
        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        service.get_settings(organization_id).await
    }

    #[transactional(doe_model, doe_settings, role, member, authz)]
    pub async fn update_doe_settings(
        &self,
        command: UpdateDoeSettingsCommand,
    ) -> Result<DoeSettings, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut service = DoeModelService::new(doe_model_repository, doe_settings_repository);
        service.update_settings(command).await
    }

    /// A sample DOE laid out with a style that is not saved yet. Gated like
    /// saving a style, since it exists to edit one.
    #[transactional(role, member, authz)]
    pub async fn preview_doe_style(
        &self,
        command: PreviewDoeStyleCommand,
    ) -> Result<DoeDocument, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "reference.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        Ok(DoeDocument::sample(
            &command.style.cleaned(),
            chrono::Utc::now().date_naive(),
        ))
    }
}
