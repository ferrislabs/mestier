use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CreateDafCommand, Daf, DafDocument, DafDossier, DafId, DafSummary, DeleteDafCommand,
    OrganizationId, PlaceDafPhotoCommand, ProjectId, UpdateDafCommand, UpdateDafDossierCommand,
    application::{MestierAuthorizer, MestierFileStorageService, MestierUseCase, policy},
    domain::{
        daf::service::{DafLookups, DafService},
        member::ports::MemberRepository,
        photo_library::service::PhotoLibraryService,
        role::ports::RoleRepository,
    },
};

#[cfg(test)]
mod tests;

async fn require_planning<M, R>(
    authz: &MestierAuthorizer,
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        "planning.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    #[transactional(daf)]
    pub async fn get_daf(
        &self,
        organization_id: OrganizationId,
        id: DafId,
    ) -> Result<Daf, CoreError> {
        DafService::new(daf_repository)
            .get_in(organization_id, id)
            .await
    }

    #[transactional(daf, project, organization, customer, doe)]
    pub async fn project_dafs(
        &self,
        organization_id: OrganizationId,
        project_id: ProjectId,
    ) -> Result<(DafDossier, Vec<DafSummary>), CoreError> {
        let mut project_repository = project_repository;
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut doe_repository = doe_repository;
        let mut lookups = DafLookups {
            projects: &mut project_repository,
            organizations: &mut organization_repository,
            customers: &mut customer_repository,
            does: &mut doe_repository,
        };
        let mut service = DafService::new(daf_repository);

        let dossier = service
            .dossier(organization_id, project_id, &mut lookups)
            .await?;
        let summaries = service.summaries(project_id).await?;
        Ok((dossier, summaries))
    }

    #[transactional(daf, project, organization, customer, doe, member, role, authz)]
    pub async fn save_daf_dossier(
        &self,
        command: UpdateDafDossierCommand,
    ) -> Result<DafDossier, CoreError> {
        let mut project_repository = project_repository;
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut doe_repository = doe_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut lookups = DafLookups {
            projects: &mut project_repository,
            organizations: &mut organization_repository,
            customers: &mut customer_repository,
            does: &mut doe_repository,
        };
        DafService::new(daf_repository)
            .save_dossier(command, &mut lookups)
            .await
    }

    #[transactional(
        daf,
        project,
        organization,
        customer,
        doe,
        product,
        supplier,
        member,
        role,
        authz
    )]
    pub async fn create_daf(&self, command: CreateDafCommand) -> Result<Daf, CoreError> {
        let mut project_repository = project_repository;
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut doe_repository = doe_repository;
        let mut product_repository = product_repository;
        let mut supplier_repository = supplier_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut lookups = DafLookups {
            projects: &mut project_repository,
            organizations: &mut organization_repository,
            customers: &mut customer_repository,
            does: &mut doe_repository,
        };
        DafService::new(daf_repository)
            .create(
                command,
                &mut lookups,
                &mut product_repository,
                &mut supplier_repository,
            )
            .await
    }

    #[transactional(daf, member, role, authz)]
    pub async fn update_daf(&self, command: UpdateDafCommand) -> Result<Daf, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut service = DafService::new(daf_repository);

        let organization_id = service.get(command.id).await?.organization_id;
        require_planning(
            authz,
            command.actor.clone(),
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        service.replace(command).await
    }

    #[transactional(daf, member, role, authz)]
    pub async fn delete_daf(&self, command: DeleteDafCommand) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut service = DafService::new(daf_repository);

        let organization_id = service.get(command.id).await?.organization_id;
        require_planning(
            authz,
            command.actor.clone(),
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        service.delete(command.id).await
    }

    #[transactional(daf, library_photo, member, role, authz)]
    pub async fn place_daf_photo(
        &self,
        command: PlaceDafPhotoCommand,
        storage: &MestierFileStorageService,
    ) -> Result<Daf, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut photos = PhotoLibraryService::new(library_photo_repository);
        DafService::new(daf_repository)
            .place_photo(command, &mut photos, storage)
            .await
    }

    #[transactional(daf, project, organization, customer, doe, doe_model, doe_settings)]
    pub async fn daf_document(
        &self,
        organization_id: OrganizationId,
        id: DafId,
    ) -> Result<DafDocument, CoreError> {
        let mut project_repository = project_repository;
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut doe_repository = doe_repository;
        let mut doe_model_repository = doe_model_repository;
        let mut doe_settings_repository = doe_settings_repository;
        let mut lookups = DafLookups {
            projects: &mut project_repository,
            organizations: &mut organization_repository,
            customers: &mut customer_repository,
            does: &mut doe_repository,
        };
        DafService::new(daf_repository)
            .document_single(
                organization_id,
                id,
                &mut lookups,
                &mut doe_model_repository,
                &mut doe_settings_repository,
            )
            .await
    }

    #[transactional(daf, project, organization, customer, doe, doe_model, doe_settings)]
    pub async fn daf_dossier_document(
        &self,
        organization_id: OrganizationId,
        project_id: ProjectId,
    ) -> Result<DafDocument, CoreError> {
        let mut project_repository = project_repository;
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut doe_repository = doe_repository;
        let mut doe_model_repository = doe_model_repository;
        let mut doe_settings_repository = doe_settings_repository;
        let mut lookups = DafLookups {
            projects: &mut project_repository,
            organizations: &mut organization_repository,
            customers: &mut customer_repository,
            does: &mut doe_repository,
        };
        DafService::new(daf_repository)
            .document_dossier(
                organization_id,
                project_id,
                &mut lookups,
                &mut doe_model_repository,
                &mut doe_settings_repository,
            )
            .await
    }
}
