use common::CoreError;
use mestier_macros::transactional;

use crate::{
    DayLog,
    application::MestierUseCase,
    domain::day_log::{commands::CloseDayCommand, service::DayLogService},
};

impl MestierUseCase {
    #[transactional(day_log, time_entry, employee)]
    pub async fn close_day(&self, command: CloseDayCommand) -> Result<DayLog, CoreError> {
        let mut service = DayLogService::new(
            day_log_repository,
            time_entry_repository,
            employee_repository,
        );
        service.close_day(command).await
    }
}
