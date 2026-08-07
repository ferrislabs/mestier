use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    TimeEntry, WorkOrderStatus,
    domain::{
        employee::ports::EmployeeRepository,
        time_entry::{
            TimeEntryId,
            commands::{
                AttachTimeEntryPhotosCommand, StartTimeEntryCommand, StopTimeEntryCommand,
            },
            ports::TimeEntryRepository,
        },
        work_order::ports::WorkOrderRepository,
    },
};

pub struct TimeEntryService<TR, WR, ER>
where
    TR: TimeEntryRepository,
    WR: WorkOrderRepository,
    ER: EmployeeRepository,
{
    time_entry_repository: TR,
    work_order_repository: WR,
    employee_repository: ER,
}

impl<TR, WR, ER> TimeEntryService<TR, WR, ER>
where
    TR: TimeEntryRepository,
    WR: WorkOrderRepository,
    ER: EmployeeRepository,
{
    pub fn new(
        time_entry_repository: TR,
        work_order_repository: WR,
        employee_repository: ER,
    ) -> Self {
        Self {
            time_entry_repository,
            work_order_repository,
            employee_repository,
        }
    }

    pub async fn start_time_entry(
        &mut self,
        command: StartTimeEntryCommand,
    ) -> Result<TimeEntry, CoreError> {
        validate_photo_keys(&command.photos_before)?;

        let employee = self
            .employee_repository
            .find_by_id(command.employee_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if employee.organization_id != command.organization_id {
            return Err(CoreError::NotFound);
        }

        let mut work_order = self
            .work_order_repository
            .find_by_id(command.work_order_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if work_order.organization_id != command.organization_id {
            return Err(CoreError::NotFound);
        }

        if matches!(
            work_order.status,
            WorkOrderStatus::Done | WorkOrderStatus::Cancelled
        ) {
            return Err(CoreError::Conflict(
                "cannot start clocking on a closed or cancelled work order".to_owned(),
            ));
        }

        let is_assigned = work_order
            .assignments
            .iter()
            .any(|assignment| assignment.employee_id == command.employee_id);
        if !is_assigned {
            return Err(CoreError::Conflict(
                "employee is not assigned to this work order".to_owned(),
            ));
        }

        if let Some(active) = self
            .time_entry_repository
            .find_active_by_employee(command.organization_id, command.employee_id)
            .await?
        {
            return Err(CoreError::Conflict(format!(
                "employee already has an open time entry ({})",
                active.id
            )));
        }

        let now = Utc::now();
        let inserted = self
            .time_entry_repository
            .insert(&TimeEntry {
                id: TimeEntryId(generate_uuid_v7()),
                organization_id: command.organization_id,
                work_order_id: command.work_order_id,
                employee_id: command.employee_id,
                started_at: now,
                ended_at: None,
                photos_before: command.photos_before,
                photos_during: Vec::new(),
                photos_after: Vec::new(),
                created_at: now,
                updated_at: now,
            })
            .await
            .map_err(map_time_entry_constraint)?;

        if work_order.status == WorkOrderStatus::Planned {
            work_order.status = WorkOrderStatus::InProgress;
            work_order.updated_at = now;
            self.work_order_repository.update(&work_order).await?;
        }

        Ok(inserted)
    }

    pub async fn stop_time_entry(
        &mut self,
        command: StopTimeEntryCommand,
    ) -> Result<TimeEntry, CoreError> {
        validate_photo_keys(&command.photos_after)?;

        let mut time_entry = self.get_time_entry(command.id).await?;
        if !time_entry.is_open() {
            return Err(CoreError::Conflict(
                "time entry is already stopped".to_owned(),
            ));
        }

        let ended_at = command.ended_at.unwrap_or_else(Utc::now);
        if ended_at <= time_entry.started_at {
            return Err(CoreError::Conflict(
                "time entry ended_at must be after started_at".to_owned(),
            ));
        }

        time_entry.ended_at = Some(ended_at);
        if !command.photos_after.is_empty() {
            time_entry.photos_after.extend(command.photos_after);
        }
        time_entry.updated_at = Utc::now();

        self.time_entry_repository.update(&time_entry).await
    }

    pub async fn attach_photos(
        &mut self,
        command: AttachTimeEntryPhotosCommand,
    ) -> Result<TimeEntry, CoreError> {
        if command.photo_keys.is_empty() {
            return Err(CoreError::Conflict(
                "photo_keys must not be empty".to_owned(),
            ));
        }
        validate_photo_keys(&command.photo_keys)?;

        let mut time_entry = self.get_time_entry(command.id).await?;
        time_entry
            .photos_mut(command.phase)
            .extend(command.photo_keys);
        time_entry.updated_at = Utc::now();

        self.time_entry_repository.update(&time_entry).await
    }

    pub async fn get_time_entry(&mut self, id: TimeEntryId) -> Result<TimeEntry, CoreError> {
        self.time_entry_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn get_active_time_entry(
        &mut self,
        organization_id: crate::OrganizationId,
        employee_id: crate::EmployeeId,
    ) -> Result<Option<TimeEntry>, CoreError> {
        self.time_entry_repository
            .find_active_by_employee(organization_id, employee_id)
            .await
    }
}

fn validate_photo_keys(keys: &[String]) -> Result<(), CoreError> {
    if keys.iter().any(|key| key.trim().is_empty()) {
        return Err(CoreError::Conflict(
            "photo keys cannot be blank".to_owned(),
        ));
    }
    Ok(())
}

fn map_time_entry_constraint(error: CoreError) -> CoreError {
    match error {
        CoreError::Conflict(constraint)
            if constraint == "uq_time_entries_one_active_per_employee" =>
        {
            CoreError::Conflict("employee already has an open time entry".to_owned())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Assignment, AssignmentId, CustomerContextId, CustomerId, Employee, EmployeeId,
        OrganizationId, TimeEntryPhotoPhase, WorkOrder, WorkOrderId,
        domain::{
            employee::ports::MockEmployeeRepository, time_entry::ports::MockTimeEntryRepository,
            work_order::ports::MockWorkOrderRepository,
        },
    };
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn service(
        time_entry_repository: MockTimeEntryRepository,
        work_order_repository: MockWorkOrderRepository,
        employee_repository: MockEmployeeRepository,
    ) -> TimeEntryService<
        MockTimeEntryRepository,
        MockWorkOrderRepository,
        MockEmployeeRepository,
    > {
        TimeEntryService::new(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        )
    }

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

    fn work_order(
        id: WorkOrderId,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
        status: WorkOrderStatus,
    ) -> WorkOrder {
        let now = Utc::now();
        WorkOrder {
            id,
            organization_id,
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            quote_id: None,
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            status,
            title: Some("Toiture".to_owned()),
            note: None,
            assignments: vec![Assignment {
                id: AssignmentId(Uuid::new_v4()),
                organization_id,
                work_order_id: id,
                employee_id,
                created_at: now,
            }],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn start_command(
        organization_id: OrganizationId,
        work_order_id: WorkOrderId,
        employee_id: EmployeeId,
    ) -> StartTimeEntryCommand {
        StartTimeEntryCommand {
            organization_id,
            work_order_id,
            employee_id,
            photos_before: Vec::new(),
        }
    }

    #[tokio::test]
    async fn start_time_entry_opens_entry_and_marks_work_order_in_progress() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let work_order_id = WorkOrderId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing_employee = employee(employee_id, organization_id);
        let existing_work_order = work_order(
            work_order_id,
            organization_id,
            employee_id,
            WorkOrderStatus::Planned,
        );

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .with(eq(employee_id))
            .returning(move |_| {
                let e = existing_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(work_order_id))
            .returning(move |_| {
                let w = existing_work_order.clone();
                Box::pin(async move { Ok(Some(w)) })
            });
        work_order_repository
            .expect_update()
            .withf(|w| w.status == WorkOrderStatus::InProgress)
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut time_entry_repository = MockTimeEntryRepository::new();
        time_entry_repository
            .expect_find_active_by_employee()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        time_entry_repository.expect_insert().returning(|entry| {
            let cloned = entry.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        );

        let created = service
            .start_time_entry(start_command(
                organization_id,
                work_order_id,
                employee_id,
            ))
            .await
            .unwrap();

        assert!(created.is_open());
        assert_eq!(created.work_order_id, work_order_id);
        assert_eq!(created.employee_id, employee_id);
    }

    #[tokio::test]
    async fn start_time_entry_rejects_when_open_entry_exists() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let work_order_id = WorkOrderId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing_employee = employee(employee_id, organization_id);
        let existing_work_order = work_order(
            work_order_id,
            organization_id,
            employee_id,
            WorkOrderStatus::InProgress,
        );
        let now = Utc::now();
        let active = TimeEntry {
            id: TimeEntryId(Uuid::new_v4()),
            organization_id,
            work_order_id,
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
        employee_repository
            .expect_find_by_id()
            .returning(move |_| {
                let e = existing_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .returning(move |_| {
                let w = existing_work_order.clone();
                Box::pin(async move { Ok(Some(w)) })
            });

        let mut time_entry_repository = MockTimeEntryRepository::new();
        time_entry_repository
            .expect_find_active_by_employee()
            .returning(move |_, _| {
                let active = active.clone();
                Box::pin(async move { Ok(Some(active)) })
            });

        let mut service = service(
            time_entry_repository,
            work_order_repository,
            employee_repository,
        );

        let err = service
            .start_time_entry(start_command(
                organization_id,
                work_order_id,
                employee_id,
            ))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn start_time_entry_rejects_unassigned_employee() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let work_order_id = WorkOrderId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let other_employee_id = EmployeeId(Uuid::new_v4());
        let existing_employee = employee(employee_id, organization_id);
        let existing_work_order = work_order(
            work_order_id,
            organization_id,
            other_employee_id,
            WorkOrderStatus::Planned,
        );

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .returning(move |_| {
                let e = existing_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .returning(move |_| {
                let w = existing_work_order.clone();
                Box::pin(async move { Ok(Some(w)) })
            });

        let mut service = service(
            MockTimeEntryRepository::new(),
            work_order_repository,
            employee_repository,
        );

        let err = service
            .start_time_entry(start_command(
                organization_id,
                work_order_id,
                employee_id,
            ))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn stop_time_entry_sets_ended_at_and_photos_after() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let id = TimeEntryId(Uuid::new_v4());
        let now = Utc::now();
        let existing = TimeEntry {
            id,
            organization_id,
            work_order_id: WorkOrderId(Uuid::new_v4()),
            employee_id: EmployeeId(Uuid::new_v4()),
            started_at: now - chrono::Duration::hours(1),
            ended_at: None,
            photos_before: Vec::new(),
            photos_during: Vec::new(),
            photos_after: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        let mut time_entry_repository = MockTimeEntryRepository::new();
        time_entry_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        time_entry_repository
            .expect_update()
            .withf(|entry| {
                entry.ended_at.is_some()
                    && entry.photos_after == vec!["uploads/after-1".to_owned()]
            })
            .returning(|entry| {
                let cloned = entry.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            time_entry_repository,
            MockWorkOrderRepository::new(),
            MockEmployeeRepository::new(),
        );

        let stopped = service
            .stop_time_entry(StopTimeEntryCommand {
                id,
                ended_at: None,
                photos_after: vec!["uploads/after-1".to_owned()],
            })
            .await
            .unwrap();

        assert!(!stopped.is_open());
        assert_eq!(stopped.photos_after, vec!["uploads/after-1".to_owned()]);
    }

    #[tokio::test]
    async fn attach_photos_appends_to_phase() {
        let id = TimeEntryId(Uuid::new_v4());
        let now = Utc::now();
        let existing = TimeEntry {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            work_order_id: WorkOrderId(Uuid::new_v4()),
            employee_id: EmployeeId(Uuid::new_v4()),
            started_at: now,
            ended_at: None,
            photos_before: vec!["uploads/before-1".to_owned()],
            photos_during: Vec::new(),
            photos_after: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        let mut time_entry_repository = MockTimeEntryRepository::new();
        time_entry_repository
            .expect_find_by_id()
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        time_entry_repository
            .expect_update()
            .withf(|entry| {
                entry.photos_during == vec!["uploads/during-1".to_owned()]
                    && entry.photos_before == vec!["uploads/before-1".to_owned()]
            })
            .returning(|entry| {
                let cloned = entry.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            time_entry_repository,
            MockWorkOrderRepository::new(),
            MockEmployeeRepository::new(),
        );

        let updated = service
            .attach_photos(AttachTimeEntryPhotosCommand {
                id,
                phase: TimeEntryPhotoPhase::During,
                photo_keys: vec!["uploads/during-1".to_owned()],
            })
            .await
            .unwrap();

        assert_eq!(updated.photos_during, vec!["uploads/during-1".to_owned()]);
    }
}
