use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    ExtraPeriod, ExtraPeriodOutcome,
    application::{MestierUseCase, leave_account::require_permission, policy},
    domain::extra_period::{
        ExtraPeriodService,
        commands::{
            CreateExtraPeriodCommand, DecideExtraPeriodCommand, ListExtraPeriodsCommand,
            RejectExtraPeriodCommand,
        },
    },
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    #[transactional(extra_period, surcharge, member, role, authz, emitter)]
    pub async fn list_extra_periods(
        &self,
        command: ListExtraPeriodsCommand,
    ) -> Result<(Vec<ExtraPeriod>, u64), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let organization_wide = policy::holds(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;
        let mut filter = command.filter;
        filter.member_id = policy::own_scope(&actor, filter.member_id, organization_wide)?;

        let mut service = ExtraPeriodService::new(
            extra_period_repository,
            surcharge_repository,
            member_repository,
            emitter,
        );
        service
            .list(
                command.organization_id,
                &filter,
                command.limit,
                command.offset,
            )
            .await
    }

    #[transactional(extra_period, surcharge, member, role, authz, emitter)]
    pub async fn create_extra_period(
        &self,
        command: CreateExtraPeriodCommand,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let creator = policy::subject_member_id(&actor);
        let privileged = policy::holds(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;
        let mut draft = command.draft;
        if !privileged {
            if creator != Some(draft.member_id) {
                return Err(CoreError::Forbidden {
                    reason: Some("a member can only declare hours for themselves".to_owned()),
                });
            }
            draft.approve = false;
            draft.rate_bp = None;
        }

        let mut service = ExtraPeriodService::new(
            extra_period_repository,
            surcharge_repository,
            member_repository,
            emitter,
        );
        service
            .create(command.organization_id, &draft, creator)
            .await
    }

    #[transactional(extra_period, surcharge, member, role, authz, emitter)]
    pub async fn approve_extra_period(
        &self,
        command: DecideExtraPeriodCommand,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let actor = require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service = ExtraPeriodService::new(
            extra_period_repository,
            surcharge_repository,
            member_repository,
            emitter,
        );
        service
            .approve(
                command.organization_id,
                command.id,
                policy::subject_member_id(&actor),
            )
            .await
    }

    #[transactional(extra_period, surcharge, member, role, authz, emitter)]
    pub async fn reject_extra_period(
        &self,
        command: RejectExtraPeriodCommand,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let actor = require_permission(
            command.actor,
            command.organization_id,
            "planning.manage",
            &mut member_repository,
            &mut role_repository,
            &authz,
        )
        .await?;

        let mut service = ExtraPeriodService::new(
            extra_period_repository,
            surcharge_repository,
            member_repository,
            emitter,
        );
        service
            .reject(
                command.organization_id,
                command.id,
                &command.comment,
                policy::subject_member_id(&actor),
            )
            .await
    }

    #[transactional(extra_period, surcharge, member, role, authz, emitter)]
    pub async fn cancel_extra_period(
        &self,
        command: DecideExtraPeriodCommand,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let privileged = match policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await
        {
            Ok(()) => true,
            Err(CoreError::Forbidden { .. }) => false,
            Err(other) => return Err(other),
        };

        let mut service = ExtraPeriodService::new(
            extra_period_repository,
            surcharge_repository,
            member_repository,
            emitter,
        );
        service
            .cancel(
                command.organization_id,
                command.id,
                policy::subject_member_id(&actor),
                privileged,
            )
            .await
    }
}
