use authz::{Resource, Subject};
use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Intervention, InterventionStatus, MemberId, OrganizationId, ProposeInterventionCommand,
    RecordInterventionCommand, RejectInterventionCommand, ValidateInterventionCommand,
    application::{
        MestierAuthorizer, MestierUseCase,
        organization::resolve_timezone,
        policy::{self, SUBJECT_MEMBER_ID_KEY},
    },
    domain::{
        maintenance::visits::InterventionService, member::ports::MemberRepository,
        organization::ports::OrganizationRepository, role::ports::RoleRepository,
    },
};

#[cfg(test)]
pub(crate) mod tests;

/// The caller's seat in `organization_id`, with `planning.manage` checked
/// when `manage` is set. Not being a member is a `Forbidden`, as everywhere.
async fn authorize<M, R>(
    authorizer: &MestierAuthorizer,
    actor: Subject,
    organization_id: OrganizationId,
    manage: bool,
    members: &mut M,
    roles: &mut R,
) -> Result<MemberId, CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    if manage {
        policy::require(
            authorizer,
            &actor,
            "planning.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await?;
    }
    actor
        .properties
        .get(SUBJECT_MEMBER_ID_KEY)
        .and_then(|value| value.as_str())
        .and_then(|raw| raw.parse().ok())
        .map(MemberId)
        .ok_or(CoreError::Forbidden {
            reason: Some("this action needs an organization member".to_owned()),
        })
}

async fn today_in(
    organizations: &mut impl OrganizationRepository,
    organization_id: OrganizationId,
) -> Result<chrono::NaiveDate, CoreError> {
    let tz = resolve_timezone(organizations, organization_id).await?;
    Ok(Utc::now().with_timezone(&tz).date_naive())
}

impl MestierUseCase {
    /// A manager's direct entry, validated at once. `NotFound` when the line
    /// is not in `organization_id`.
    #[transactional(
        maintenance_contract,
        maintenance_intervention,
        organization,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn record_maintenance_intervention(
        &self,
        organization_id: OrganizationId,
        command: RecordInterventionCommand,
    ) -> Result<Intervention, CoreError> {
        let mut organization_repository = organization_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let by = authorize(
            authz,
            command.actor,
            organization_id,
            true,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let today = today_in(&mut organization_repository, organization_id).await?;

        let mut service = InterventionService::new(
            maintenance_contract_repository,
            maintenance_intervention_repository,
            emitter,
        );
        let contract = service
            .contract_of_line(organization_id, command.line_id)
            .await?;
        service
            .record(&contract, command.line_id, by, command.done_on, today)
            .await
    }

    /// A field worker's date on one of their own maintenance tasks. `NotFound`
    /// for any task they are not assigned to.
    #[transactional(
        maintenance_contract,
        maintenance_intervention,
        organization,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn propose_maintenance_intervention(
        &self,
        command: ProposeInterventionCommand,
    ) -> Result<Intervention, CoreError> {
        let mut organization_repository = organization_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let by = authorize(
            authz,
            command.actor,
            command.organization_id,
            false,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let today = today_in(&mut organization_repository, command.organization_id).await?;

        InterventionService::new(
            maintenance_contract_repository,
            maintenance_intervention_repository,
            emitter,
        )
        .propose(
            command.organization_id,
            command.task_id,
            by,
            command.done_on,
            today,
        )
        .await
    }

    /// `NotFound` for an intervention of another organization than
    /// `organization_id`, `Conflict` for one already resolved.
    #[transactional(
        maintenance_contract,
        maintenance_intervention,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn validate_maintenance_intervention(
        &self,
        organization_id: OrganizationId,
        command: ValidateInterventionCommand,
    ) -> Result<Intervention, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let by = authorize(
            authz,
            command.actor,
            organization_id,
            true,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut service = InterventionService::new(
            maintenance_contract_repository,
            maintenance_intervention_repository,
            emitter,
        );
        let intervention = service.get(organization_id, command.id).await?;
        service.validate(intervention, by).await
    }

    #[transactional(
        maintenance_contract,
        maintenance_intervention,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn reject_maintenance_intervention(
        &self,
        organization_id: OrganizationId,
        command: RejectInterventionCommand,
    ) -> Result<Intervention, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let by = authorize(
            authz,
            command.actor,
            organization_id,
            true,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut service = InterventionService::new(
            maintenance_contract_repository,
            maintenance_intervention_repository,
            emitter,
        );
        let intervention = service.get(organization_id, command.id).await?;
        service.reject(intervention, by, command.note).await
    }

    /// A page of `organization_id`'s interventions in `status`, oldest first.
    #[transactional(
        maintenance_contract,
        maintenance_intervention,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn list_maintenance_interventions(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        status: InterventionStatus,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Intervention>, u64), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        authorize(
            authz,
            actor,
            organization_id,
            true,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        InterventionService::new(
            maintenance_contract_repository,
            maintenance_intervention_repository,
            emitter,
        )
        .list_by_status(organization_id, status, limit, offset)
        .await
    }
}
