use common::CoreError;
use mestier_macros::transactional;

use crate::{
    DateRange, EmployeeId, EmployeeRhythm, EmployeeWorkSlot,
    application::MestierUseCase,
    domain::work_time::{
        commands::{ReplaceRhythmCommand, ReplaceWorkSlotsCommand},
        service::WorkTimeService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(employee_rhythm, employee_work_slot)]
    pub async fn get_work_time(
        &self,
        employee_id: EmployeeId,
        range: DateRange,
    ) -> Result<(Vec<EmployeeRhythm>, Vec<EmployeeWorkSlot>), CoreError> {
        let mut service =
            WorkTimeService::new(employee_rhythm_repository, employee_work_slot_repository);
        service.get_work_time(employee_id, range).await
    }

    /// Replaces the rhythm currently in effect wholesale — see
    /// `WorkTimeService::replace_rhythm` for the full-replace-vs-new-version
    /// rules.
    #[transactional(employee_rhythm, employee_work_slot)]
    pub async fn replace_rhythm(
        &self,
        command: ReplaceRhythmCommand,
    ) -> Result<EmployeeRhythm, CoreError> {
        let mut service =
            WorkTimeService::new(employee_rhythm_repository, employee_work_slot_repository);
        service.replace_rhythm(command).await
    }

    /// Replaces the `[from, to]` work-slots window wholesale.
    #[transactional(employee_rhythm, employee_work_slot)]
    pub async fn replace_work_slots(
        &self,
        command: ReplaceWorkSlotsCommand,
    ) -> Result<Vec<EmployeeWorkSlot>, CoreError> {
        let mut service =
            WorkTimeService::new(employee_rhythm_repository, employee_work_slot_repository);
        service.replace_work_slots(command).await
    }
}
