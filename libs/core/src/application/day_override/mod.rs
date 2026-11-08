use common::CoreError;
use mestier_macros::transactional;

use crate::{
    DayOverride, DayOverrideOutcome, FeedMember, HrPlanningFeed,
    application::{MestierUseCase, leave_account::require_permission},
    domain::{
        absence_type::AbsenceTypeService,
        day_override::{
            DayOverrideService,
            commands::{
                DeleteDayOverrideCommand, HrPlanningFeedCommand, ListDayOverridesCommand,
                PutDayOverrideCommand,
            },
        },
        member::ports::MemberRepository,
        member_contract::ports::MemberContractRepository,
        member_profile::ports::MemberProfileRepository,
    },
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    #[transactional(day_override, member, role, authz)]
    pub async fn list_day_overrides(
        &self,
        command: ListDayOverridesCommand,
    ) -> Result<Vec<DayOverride>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service = DayOverrideService::new(day_override_repository, member_repository);
        service
            .list(command.organization_id, command.period, command.member_id)
            .await
    }

    #[transactional(day_override, member, role, authz)]
    pub async fn put_day_override(
        &self,
        command: PutDayOverrideCommand,
    ) -> Result<DayOverrideOutcome<DayOverride>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service = DayOverrideService::new(day_override_repository, member_repository);
        service
            .put(
                command.organization_id,
                command.member_id,
                command.date,
                &command.input,
            )
            .await
    }

    #[transactional(day_override, member, role, authz)]
    pub async fn delete_day_override(
        &self,
        command: DeleteDayOverrideCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service = DayOverrideService::new(day_override_repository, member_repository);
        service
            .delete(command.organization_id, command.member_id, command.date)
            .await
    }

    #[transactional(
        day_override,
        member,
        member_profile,
        member_contract,
        absence,
        absence_type,
        leave_account,
        role,
        authz
    )]
    pub async fn hr_planning_feed(
        &self,
        command: HrPlanningFeedCommand,
    ) -> Result<HrPlanningFeed, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut day_override_repository = day_override_repository;
        let mut member_profile_repository = member_profile_repository;
        let mut member_contract_repository = member_contract_repository;
        let mut absence_repository = absence_repository;
        require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let organization_id = command.organization_id;
        let period = command.period;
        let members = member_repository
            .list_with_archived_by_organization(organization_id)
            .await?;
        let archived: std::collections::HashSet<_> = member_profile_repository
            .list_by_organization(organization_id)
            .await?
            .into_iter()
            .filter(|profile| profile.archived_at.is_some())
            .map(|profile| profile.member_id)
            .collect();
        let contracts = member_contract_repository
            .list_overlapping(organization_id, period.from(), period.to())
            .await?;
        let absences = crate::domain::absence::ports::AbsenceRepository::list_overlapping(
            &mut absence_repository,
            organization_id,
            period.from(),
            period.to(),
        )
        .await?;
        let overrides = crate::domain::day_override::ports::DayOverrideRepository::list(
            &mut day_override_repository,
            organization_id,
            period,
            None,
        )
        .await?;
        let absence_types =
            AbsenceTypeService::new(absence_type_repository, leave_account_repository)
                .list(organization_id)
                .await?;

        Ok(HrPlanningFeed {
            members: members
                .into_iter()
                .map(|member| FeedMember {
                    archived: archived.contains(&member.id),
                    member,
                })
                .collect(),
            contracts,
            absences,
            overrides,
            absence_types,
        })
    }
}
