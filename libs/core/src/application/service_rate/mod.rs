use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, ServiceRate, ServiceRateId,
    application::{MestierUseCase, policy},
    domain::service_rate::{
        commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
        service::ServiceRateService,
    },
};

impl MestierUseCase {
    #[transactional(service_rate, role, member, authz)]
    pub async fn create_service_rate(
        &self,
        command: CreateServiceRateCommand,
    ) -> Result<ServiceRate, CoreError> {
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

        let mut service = ServiceRateService::new(service_rate_repository);
        service.create_service_rate(command).await
    }

    #[transactional(service_rate)]
    pub async fn get_service_rate(&self, id: ServiceRateId) -> Result<ServiceRate, CoreError> {
        let mut service = ServiceRateService::new(service_rate_repository);
        service.get_service_rate(id).await
    }

    #[transactional(service_rate)]
    pub async fn list_service_rates(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ServiceRate>, u64), CoreError> {
        let mut service = ServiceRateService::new(service_rate_repository);
        service
            .list_service_rates(organization_id, limit, offset)
            .await
    }

    /// The service rate row is loaded first and authorization runs against
    /// *its own* `organization_id`, never one taken from the request path —
    /// a bare `/service-rates/{id}` route has no organization to trust
    /// otherwise (CLAUDE.md's "bare ids derive their organization from the
    /// loaded row" rule).
    #[transactional(service_rate, role, member, authz)]
    pub async fn update_service_rate(
        &self,
        command: UpdateServiceRateCommand,
    ) -> Result<ServiceRate, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = ServiceRateService::new(service_rate_repository);
        let existing = service.get_service_rate(command.id).await?;

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

        service.update_service_rate(command).await
    }

    /// Same "load, then authorize against the loaded row's own organization"
    /// rule as [`Self::update_service_rate`] — there is no domain command to
    /// carry an `actor` for a bare-id delete, so it is threaded as its own
    /// parameter instead, the same way `remove_employee_profile` does.
    #[transactional(service_rate, role, member, authz)]
    pub async fn soft_delete_service_rate(
        &self,
        actor: Subject,
        id: ServiceRateId,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = ServiceRateService::new(service_rate_repository);
        let existing = service.get_service_rate(id).await?;

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

        service.soft_delete_service_rate(id).await
    }
}
