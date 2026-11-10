use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    DoeDocument, DoeModel, DoeModelId, DoeSettings, DoeStyle, OrganizationId, ProjectDoeStyle,
    ProjectId,
    application::{MestierAuthorizer, MestierUseCase, policy},
    domain::{
        doe::ports::DoeRepository,
        doe_model::{
            commands::{
                CreateDoeModelCommand, CreateDoeModelFromStarterCommand, PreviewDoeStyleCommand,
                ProjectDoeStyleCommand, UpdateDoeModelCommand, UpdateDoeSettingsCommand,
                UpdateProjectDoeStyleCommand,
            },
            ports::DoeSettingsRepository,
            service::{DoeModelService, inherited_style},
        },
        member::ports::MemberRepository,
        project::ports::ProjectRepository,
        role::ports::RoleRepository,
    },
};

/// A project's style is edited like its DOE: with `planning.manage`, on a
/// project of the organization.
async fn require_project_style<M, R, P>(
    authz: &MestierAuthorizer,
    actor: authz::Subject,
    organization_id: OrganizationId,
    project_id: ProjectId,
    members: &mut M,
    roles: &mut R,
    projects: &mut P,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
    P: ProjectRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        "planning.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await?;
    projects
        .find_by_id(project_id)
        .await?
        .filter(|project| project.organization_id == organization_id)
        .map(|_| ())
        .ok_or(CoreError::NotFound)
}

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
    /// saving a style, the organization's or a project's, since it exists to
    /// edit one.
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
        let organization = Resource::new("organization", command.organization_id.0.to_string());
        if !policy::holds(&authz, &actor, "reference.manage", organization.clone()).await? {
            policy::require(&authz, &actor, "planning.manage", organization).await?;
        }

        Ok(DoeDocument::sample(
            &command.style.cleaned(),
            chrono::Utc::now().date_naive(),
        ))
    }

    #[transactional(project, doe, doe_model, doe_settings, role, member, authz)]
    pub async fn get_project_doe_style(
        &self,
        command: ProjectDoeStyleCommand,
    ) -> Result<ProjectDoeStyle, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut project_repository = project_repository;
        let mut doe_repository = doe_repository;
        let mut doe_model_repository = doe_model_repository;
        let mut doe_settings_repository = doe_settings_repository;
        require_project_style(
            authz,
            command.actor,
            command.organization_id,
            command.project_id,
            &mut member_repository,
            &mut role_repository,
            &mut project_repository,
        )
        .await?;

        let model_id = doe_repository
            .find_by_project(command.project_id)
            .await?
            .filter(|doe| doe.organization_id == command.organization_id)
            .and_then(|doe| doe.model_id);
        let inherited = inherited_style(
            &mut doe_model_repository,
            &mut doe_settings_repository,
            command.organization_id,
            model_id,
        )
        .await?;
        let style = doe_settings_repository
            .find_project_style(command.organization_id, command.project_id)
            .await?;
        Ok(ProjectDoeStyle { style, inherited })
    }

    #[transactional(project, doe_settings, role, member, authz)]
    pub async fn update_project_doe_style(
        &self,
        command: UpdateProjectDoeStyleCommand,
    ) -> Result<DoeStyle, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut project_repository = project_repository;
        let mut doe_settings_repository = doe_settings_repository;
        require_project_style(
            authz,
            command.actor,
            command.organization_id,
            command.project_id,
            &mut member_repository,
            &mut role_repository,
            &mut project_repository,
        )
        .await?;

        doe_settings_repository
            .upsert_project_style(
                command.organization_id,
                command.project_id,
                &command.style.cleaned(),
            )
            .await
    }

    #[transactional(project, doe_settings, role, member, authz)]
    pub async fn clear_project_doe_style(
        &self,
        command: ProjectDoeStyleCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut project_repository = project_repository;
        let mut doe_settings_repository = doe_settings_repository;
        require_project_style(
            authz,
            command.actor,
            command.organization_id,
            command.project_id,
            &mut member_repository,
            &mut role_repository,
            &mut project_repository,
        )
        .await?;

        doe_settings_repository
            .delete_project_style(command.organization_id, command.project_id)
            .await
    }
}
