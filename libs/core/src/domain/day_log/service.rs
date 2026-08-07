use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    DayLog,
    domain::{
        day_log::{
            DayLogId,
            commands::CloseDayCommand,
            ports::DayLogRepository,
        },
        employee::ports::EmployeeRepository,
        time_entry::ports::TimeEntryRepository,
    },
};

pub struct DayLogService<DR, TR, ER>
where
    DR: DayLogRepository,
    TR: TimeEntryRepository,
    ER: EmployeeRepository,
{
    day_log_repository: DR,
    time_entry_repository: TR,
    employee_repository: ER,
}

impl<DR, TR, ER> DayLogService<DR, TR, ER>
where
    DR: DayLogRepository,
    TR: TimeEntryRepository,
    ER: EmployeeRepository,
{
    pub fn new(
        day_log_repository: DR,
        time_entry_repository: TR,
        employee_repository: ER,
    ) -> Self {
        Self {
            day_log_repository,
            time_entry_repository,
            employee_repository,
        }
    }

    /// Declares end of day. Refuses (409) if the employee still has an open
    /// time entry — the chantier must be stopped first; we never auto-close.
    pub async fn close_day(&mut self, command: CloseDayCommand) -> Result<DayLog, CoreError> {
        let employee = self
            .employee_repository
            .find_by_id(command.employee_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if employee.organization_id != command.organization_id {
            return Err(CoreError::NotFound);
        }

        if let Some(active) = self
            .time_entry_repository
            .find_active_by_employee(command.organization_id, command.employee_id)
            .await?
        {
            return Err(CoreError::Conflict(format!(
                "cannot close day while time entry {} is still open",
                active.id
            )));
        }

        let now = Utc::now();
        let ended_at = command.ended_at.unwrap_or(now);

        self.day_log_repository
            .insert(&DayLog {
                id: DayLogId(generate_uuid_v7()),
                organization_id: command.organization_id,
                employee_id: command.employee_id,
                work_date: command.work_date,
                ended_at,
                created_at: now,
            })
            .await
            .map_err(map_day_log_constraint)
    }
}

fn map_day_log_constraint(error: CoreError) -> CoreError {
    match error {
        CoreError::Conflict(constraint) if constraint == "uq_day_logs_employee_work_date" => {
            CoreError::Conflict("day log already exists for this employee and date".to_owned())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Employee, EmployeeId, OrganizationId, TimeEntry, TimeEntryId, WorkOrderId,
        domain::{
            day_log::ports::MockDayLogRepository, employee::ports::MockEmployeeRepository,
            time_entry::ports::MockTimeEntryRepository,
        },
    };
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn employee(id: EmployeeId, organization_id: OrganizationId) -> Employee {
        let now = Utc::now();
        Employee {
            id,
            organization_id,
            user_id: None,
            name: "Alice".to_owned(),
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn close_day_persists_day_log_when_no_open_entry() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing = employee(employee_id, organization_id);
        let work_date = NaiveDate::from_ymd_opt(2026, 8, 7).unwrap();

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository.expect_find_by_id().returning(move |_| {
            let e = existing.clone();
            Box::pin(async move { Ok(Some(e)) })
        });

        let mut time_entry_repository = MockTimeEntryRepository::new();
        time_entry_repository
            .expect_find_active_by_employee()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let mut day_log_repository = MockDayLogRepository::new();
        day_log_repository.expect_insert().returning(|day_log| {
            let cloned = day_log.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = DayLogService::new(
            day_log_repository,
            time_entry_repository,
            employee_repository,
        );

        let created = service
            .close_day(CloseDayCommand {
                organization_id,
                employee_id,
                work_date,
                ended_at: None,
            })
            .await
            .unwrap();

        assert_eq!(created.work_date, work_date);
        assert_eq!(created.employee_id, employee_id);
    }

    #[tokio::test]
    async fn close_day_rejects_open_time_entry() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing = employee(employee_id, organization_id);
        let now = Utc::now();
        let active = TimeEntry {
            id: TimeEntryId(Uuid::new_v4()),
            organization_id,
            work_order_id: WorkOrderId(Uuid::new_v4()),
            employee_id,
            started_at: now,
            ended_at: None,
            photos_before: Vec::new(),
            photos_during: Vec::new(),
            photos_after: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository.expect_find_by_id().returning(move |_| {
            let e = existing.clone();
            Box::pin(async move { Ok(Some(e)) })
        });

        let mut time_entry_repository = MockTimeEntryRepository::new();
        time_entry_repository
            .expect_find_active_by_employee()
            .returning(move |_, _| {
                let active = active.clone();
                Box::pin(async move { Ok(Some(active)) })
            });

        let mut service = DayLogService::new(
            MockDayLogRepository::new(),
            time_entry_repository,
            employee_repository,
        );

        let err = service
            .close_day(CloseDayCommand {
                organization_id,
                employee_id,
                work_date: NaiveDate::from_ymd_opt(2026, 8, 7).unwrap(),
                ended_at: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }
}
