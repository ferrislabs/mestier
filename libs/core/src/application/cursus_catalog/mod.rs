use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    ClosureReason, ClosureReasonId, Establishment, EstablishmentId, OrganizationId, Promotion,
    PromotionId, Training, TrainingId,
    application::{MestierUseCase, policy},
    domain::cursus_catalog::{
        commands::{
            CreateClosureReasonCommand, CreateEstablishmentCommand, CreatePromotionCommand,
            CreateTrainingCommand, UpdateClosureReasonCommand, UpdateEstablishmentCommand,
            UpdatePromotionCommand, UpdateTrainingCommand,
        },
        service::CursusCatalogService,
    },
};

#[cfg(test)]
mod tests;

async fn require_catalog_manager<M, R, A>(
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
    authz: &A,
) -> Result<(), CoreError>
where
    M: crate::domain::member::ports::MemberRepository,
    R: crate::domain::role::ports::RoleRepository,
    A: authz::Authorizer,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        "cursus.catalog.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn create_establishment(
        &self,
        command: CreateEstablishmentCommand,
    ) -> Result<Establishment, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .create_establishment(command)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn get_establishment(
        &self,
        organization_id: OrganizationId,
        id: EstablishmentId,
    ) -> Result<Establishment, CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .get_establishment(organization_id, id)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn list_establishments(
        &self,
        organization_id: OrganizationId,
        include_archived: bool,
    ) -> Result<Vec<Establishment>, CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .list_establishments(organization_id, include_archived)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn update_establishment(
        &self,
        command: UpdateEstablishmentCommand,
    ) -> Result<Establishment, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .update_establishment(command)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn archive_establishment(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: EstablishmentId,
    ) -> Result<Establishment, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .archive_establishment(organization_id, id)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn create_training(
        &self,
        command: CreateTrainingCommand,
    ) -> Result<Training, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .create_training(command)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn get_training(
        &self,
        organization_id: OrganizationId,
        id: TrainingId,
    ) -> Result<Training, CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .get_training(organization_id, id)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn list_trainings(
        &self,
        organization_id: OrganizationId,
        establishment_id: Option<EstablishmentId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Training>, u64), CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .list_trainings(
                organization_id,
                establishment_id,
                include_archived,
                limit,
                offset,
            )
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn update_training(
        &self,
        command: UpdateTrainingCommand,
    ) -> Result<Training, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .update_training(command)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn archive_training(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: TrainingId,
    ) -> Result<Training, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .archive_training(organization_id, id)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn create_promotion(
        &self,
        command: CreatePromotionCommand,
    ) -> Result<Promotion, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .create_promotion(command)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn get_promotion(
        &self,
        organization_id: OrganizationId,
        id: PromotionId,
    ) -> Result<Promotion, CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .get_promotion(organization_id, id)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn list_promotions(
        &self,
        organization_id: OrganizationId,
        training_id: Option<TrainingId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Promotion>, u64), CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .list_promotions(
                organization_id,
                training_id,
                include_archived,
                limit,
                offset,
            )
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn update_promotion(
        &self,
        command: UpdatePromotionCommand,
    ) -> Result<Promotion, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .update_promotion(command)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn archive_promotion(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: PromotionId,
    ) -> Result<Promotion, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .archive_promotion(organization_id, id)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn create_closure_reason(
        &self,
        command: CreateClosureReasonCommand,
    ) -> Result<ClosureReason, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .create_closure_reason(command)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn get_closure_reason(
        &self,
        organization_id: OrganizationId,
        id: ClosureReasonId,
    ) -> Result<ClosureReason, CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .get_closure_reason(organization_id, id)
            .await
    }

    #[transactional(cursus_catalog)]
    pub async fn list_closure_reasons(
        &self,
        organization_id: OrganizationId,
        include_archived: bool,
    ) -> Result<Vec<ClosureReason>, CoreError> {
        CursusCatalogService::new(cursus_catalog_repository)
            .list_closure_reasons(organization_id, include_archived)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn update_closure_reason(
        &self,
        command: UpdateClosureReasonCommand,
    ) -> Result<ClosureReason, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .update_closure_reason(command)
            .await
    }

    #[transactional(cursus_catalog, role, member, authz)]
    pub async fn archive_closure_reason(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        id: ClosureReasonId,
    ) -> Result<ClosureReason, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_catalog_manager(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        CursusCatalogService::new(cursus_catalog_repository)
            .archive_closure_reason(organization_id, id)
            .await
    }
}
