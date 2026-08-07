use common::CoreError;
use mestier_macros::transactional;

use crate::{
    EmployeeId, OrganizationId, TimeEntry, TimeEntryId,
    application::MestierUseCase,
    domain::time_entry::{
        commands::{
            AttachTimeEntryPhotosCommand, StartTimeEntryCommand, StopTimeEntryCommand,
        },
        service::TimeEntryService,
    },
};

impl MestierUseCase {
    #[transactional(time_entry, work_order, employee)]
    pub async fn start_time_entry(
        &self,
        command: StartTimeEntryCommand,
    ) -> Result<TimeEntry, CoreError> {
        let mut service = TimeEntryService::new(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        );
        service.start_time_entry(command).await
    }

    #[transactional(time_entry, work_order, employee)]
    pub async fn stop_time_entry(
        &self,
        command: StopTimeEntryCommand,
    ) -> Result<TimeEntry, CoreError> {
        let mut service = TimeEntryService::new(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        );
        service.stop_time_entry(command).await
    }

    #[transactional(time_entry, work_order, employee)]
    pub async fn attach_time_entry_photos(
        &self,
        command: AttachTimeEntryPhotosCommand,
    ) -> Result<TimeEntry, CoreError> {
        let mut service = TimeEntryService::new(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        );
        service.attach_photos(command).await
    }

    #[transactional(time_entry, work_order, employee)]
    pub async fn get_time_entry(&self, id: TimeEntryId) -> Result<TimeEntry, CoreError> {
        let mut service = TimeEntryService::new(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        );
        service.get_time_entry(id).await
    }

    #[transactional(time_entry, work_order, employee)]
    pub async fn get_active_time_entry(
        &self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
    ) -> Result<Option<TimeEntry>, CoreError> {
        let mut service = TimeEntryService::new(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        );
        service
            .get_active_time_entry(organization_id, employee_id)
            .await
    }
}
