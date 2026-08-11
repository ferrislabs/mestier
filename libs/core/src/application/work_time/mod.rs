use common::CoreError;
use mestier_macros::transactional;

use chrono::NaiveDate;

use crate::{
    DateRange, EmployeeRhythm, MemberId, OrganizationId, WorkSlot,
    application::MestierUseCase,
    domain::{
        employee::ports::EmployeeRepository,
        work_time::{
            commands::{ReplaceRhythmCommand, ReplaceWorkSlotsCommand, RhythmSlotInput},
            service::WorkTimeService,
        },
    },
};

mod tests;

impl MestierUseCase {
    /// Addressed by member, like every other route on a person.
    ///
    /// The rhythm is the one part still keyed on the contract, so the profile
    /// is resolved here rather than asked of the caller — who has no reason to
    /// know whether the member has one.
    #[transactional(employee_rhythm, work_slot, employee)]
    pub async fn get_work_time(
        &self,
        member_id: MemberId,
        range: DateRange,
    ) -> Result<(Vec<EmployeeRhythm>, Vec<WorkSlot>), CoreError> {
        let mut employee_repository = employee_repository;
        let employee_id = employee_repository
            .find_by_member_id(member_id)
            .await?
            .map(|profile| profile.id);

        let mut service = WorkTimeService::new(employee_rhythm_repository, work_slot_repository);
        service.get_work_time(member_id, employee_id, range).await
    }

    /// Replaces the rhythm currently in effect wholesale — see
    /// `WorkTimeService::replace_rhythm` for the full-replace-vs-new-version
    /// rules.
    ///
    /// Addressed by member, like every route on a person, but a rhythm is the
    /// translation of a *contract* into recurring slots: a member with no
    /// profile has nothing to translate, and that is a `NotFound` rather than
    /// an empty rhythm silently created on their behalf.
    #[transactional(employee_rhythm, work_slot, employee)]
    pub async fn replace_rhythm(
        &self,
        organization_id: OrganizationId,
        member_id: MemberId,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        slots: Vec<RhythmSlotInput>,
    ) -> Result<EmployeeRhythm, CoreError> {
        let mut employee_repository = employee_repository;
        let employee_id = employee_repository
            .find_by_member_id(member_id)
            .await?
            .map(|profile| profile.id)
            .ok_or(CoreError::NotFound)?;

        let mut service = WorkTimeService::new(employee_rhythm_repository, work_slot_repository);
        service
            .replace_rhythm(ReplaceRhythmCommand {
                organization_id,
                employee_id,
                effective_from,
                effective_to,
                slots,
            })
            .await
    }

    /// Replaces the `[from, to]` work-slots window wholesale.
    #[transactional(employee_rhythm, work_slot)]
    pub async fn replace_work_slots(
        &self,
        command: ReplaceWorkSlotsCommand,
    ) -> Result<Vec<WorkSlot>, CoreError> {
        let mut service = WorkTimeService::new(employee_rhythm_repository, work_slot_repository);
        service.replace_work_slots(command).await
    }
}
