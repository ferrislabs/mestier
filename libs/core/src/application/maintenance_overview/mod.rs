use authz::{Resource, Subject};
use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Grid, IsoWeekNumber, OrganizationId, PlanMaintenanceLinesCommand, Recap, Task,
    application::{MestierAuthorizer, MestierUseCase, organization::resolve_timezone, policy},
    domain::{
        maintenance::{
            overview::{build_grid, build_recap, load_season, load_sites, monday_of_week},
            plan::PlanService,
        },
        member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

#[cfg(test)]
mod tests;

/// Both views carry the conducteur's whole book of work, so reading them
/// needs `planning.manage` like writing does.
async fn require_planning_manage<M, R>(
    authorizer: &MestierAuthorizer,
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
        authorizer,
        &actor,
        "planning.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    /// The season grid: every active contract of `year`, its lines, their
    /// visits and their due date as of today in the organization's timezone.
    #[transactional(
        maintenance_contract,
        maintenance_intervention,
        customer,
        customer_context,
        customer_contact,
        organization,
        member,
        role,
        authz
    )]
    pub async fn maintenance_grid(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        year: i32,
    ) -> Result<Grid, CoreError> {
        let mut maintenance_contract_repository = maintenance_contract_repository;
        let mut maintenance_intervention_repository = maintenance_intervention_repository;
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut customer_contact_repository = customer_contact_repository;
        let mut organization_repository = organization_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        require_planning_manage(
            authz,
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let tz = resolve_timezone(&mut organization_repository, organization_id).await?;
        let today = Utc::now().with_timezone(&tz).date_naive();
        let data = load_season(
            &mut maintenance_contract_repository,
            &mut maintenance_intervention_repository,
            organization_id,
            year,
        )
        .await?;
        let sites = load_sites(
            &data.contracts,
            &mut customer_repository,
            &mut customer_context_repository,
            &mut customer_contact_repository,
        )
        .await?;

        build_grid(year, &data, &sites, today)
    }

    /// What to schedule for ISO week `week` of `year`: the lines due in it
    /// and the overdue ones, grouped by town.
    #[transactional(
        maintenance_contract,
        maintenance_intervention,
        customer,
        customer_context,
        customer_contact,
        member,
        role,
        authz
    )]
    pub async fn maintenance_recap(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        year: i32,
        week: IsoWeekNumber,
    ) -> Result<Recap, CoreError> {
        let mut maintenance_contract_repository = maintenance_contract_repository;
        let mut maintenance_intervention_repository = maintenance_intervention_repository;
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut customer_contact_repository = customer_contact_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        require_planning_manage(
            authz,
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        monday_of_week(year, week)?;
        let data = load_season(
            &mut maintenance_contract_repository,
            &mut maintenance_intervention_repository,
            organization_id,
            year,
        )
        .await?;
        let sites = load_sites(
            &data.contracts,
            &mut customer_repository,
            &mut customer_context_repository,
            &mut customer_contact_repository,
        )
        .await?;

        build_recap(year, week, &data, &sites)
    }

    /// One undated backlog task per line, in the contract's project. All or
    /// nothing.
    #[transactional(maintenance_contract, task, member, role, authz)]
    pub async fn plan_maintenance_lines(
        &self,
        command: PlanMaintenanceLinesCommand,
    ) -> Result<Vec<Task>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        require_planning_manage(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PlanService::new(maintenance_contract_repository, task_repository)
            .plan(
                command.organization_id,
                &command.line_ids,
                command.year,
                command.week,
            )
            .await
    }
}
