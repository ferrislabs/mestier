use std::collections::HashSet;

use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Employee, EmployeeId, WorkOrder,
    domain::{
        employee::ports::EmployeeRepository,
        member::ports::MemberRepository,
        user::ports::UserRepository,
        work_order::{
            Assignment, AssignmentId, WorkOrderId,
            commands::{CreateWorkOrderCommand, PatchWorkOrderCommand},
            ports::WorkOrderRepository,
        },
    },
};

/// Orchestrates work orders together with the employee, user and member
/// repositories it needs to resolve `PATCH`'s `assignees` — in particular
/// the on-the-fly employee creation for a `member` assignee who does not
/// have an employee record yet. This mirrors how `OrganizationService`
/// composes `role`/`member`/`user` repositories directly: cross-aggregate
/// orchestration lives in the domain service that owns the use case, never
/// in the thin `#[transactional]` application layer.
pub struct WorkOrderService<WR, ER, UR, MR>
where
    WR: WorkOrderRepository,
    ER: EmployeeRepository,
    UR: UserRepository,
    MR: MemberRepository,
{
    work_order_repository: WR,
    employee_repository: ER,
    user_repository: UR,
    member_repository: MR,
}

impl<WR, ER, UR, MR> WorkOrderService<WR, ER, UR, MR>
where
    WR: WorkOrderRepository,
    ER: EmployeeRepository,
    UR: UserRepository,
    MR: MemberRepository,
{
    pub fn new(
        work_order_repository: WR,
        employee_repository: ER,
        user_repository: UR,
        member_repository: MR,
    ) -> Self {
        Self {
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        }
    }

    pub async fn create_work_order(
        &mut self,
        command: CreateWorkOrderCommand,
    ) -> Result<WorkOrder, CoreError> {
        validate_schedule(command.starts_at, command.ends_at)?;
        validate_text_field("work order title", &command.title)?;
        validate_text_field("work order note", &command.note)?;

        let now = Utc::now();
        self.work_order_repository
            .insert(&WorkOrder {
                id: WorkOrderId(generate_uuid_v7()),
                organization_id: command.organization_id,
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                quote_id: command.quote_id,
                starts_at: command.starts_at,
                ends_at: command.ends_at,
                all_day: command.all_day,
                status: crate::WorkOrderStatus::Planned,
                title: command.title,
                note: command.note,
                assignments: Vec::new(),
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_work_order(&mut self, id: WorkOrderId) -> Result<WorkOrder, CoreError> {
        self.work_order_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_work_orders(
        &mut self,
        organization_id: crate::OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<WorkOrder>, u64), CoreError> {
        self.work_order_repository
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    /// Applies a `PATCH`: reschedule, status/title/note edits, and a full
    /// assignee replacement, all against the single `WorkOrder` loaded at
    /// the top — the caller wraps this in one transaction (see
    /// `#[transactional(work_order, employee, user, member)]` on
    /// `MestierUseCase::patch_work_order`), so either every write here lands
    /// or none does.
    ///
    /// Returns the updated work order together with the employee records
    /// created on the fly for `member` assignees who had none yet.
    pub async fn patch_work_order(
        &mut self,
        command: PatchWorkOrderCommand,
    ) -> Result<(WorkOrder, Vec<Employee>), CoreError> {
        let mut work_order = self.get_work_order(command.id).await?;

        let starts_at = command.starts_at.unwrap_or(work_order.starts_at);
        let ends_at = command.ends_at.unwrap_or(work_order.ends_at);
        validate_schedule(starts_at, ends_at)?;

        let title = command.title.unwrap_or_else(|| work_order.title.clone());
        let note = command.note.unwrap_or_else(|| work_order.note.clone());
        validate_text_field("work order title", &title)?;
        validate_text_field("work order note", &note)?;

        work_order.starts_at = starts_at;
        work_order.ends_at = ends_at;
        if let Some(all_day) = command.all_day {
            work_order.all_day = all_day;
        }
        if let Some(status) = command.status {
            work_order.status = status;
        }
        work_order.title = title;
        work_order.note = note;
        work_order.updated_at = Utc::now();

        let created_employees = if let Some(assignees) = command.assignees {
            let (assignments, created_employees) =
                self.resolve_assignments(&work_order, assignees).await?;
            work_order.assignments = assignments;
            created_employees
        } else {
            Vec::new()
        };

        let updated = self.work_order_repository.update(&work_order).await?;
        Ok((updated, created_employees))
    }

    /// Resolves each `AssigneeRef` to a concrete, deduplicated `employee_id`
    /// list, provisioning an employee record on the fly for `member`
    /// assignees who don't have one yet (`hourly_rate_cents` stays `NULL`
    /// — never defaulted to `0`, which would read as "genuinely free").
    async fn resolve_assignments(
        &mut self,
        work_order: &WorkOrder,
        assignees: Vec<crate::AssigneeRef>,
    ) -> Result<(Vec<Assignment>, Vec<Employee>), CoreError> {
        let mut created_employees = Vec::new();
        let mut seen = HashSet::new();
        let mut employee_ids = Vec::with_capacity(assignees.len());

        for assignee in assignees {
            let employee_id = self
                .resolve_assignee(work_order, assignee, &mut created_employees)
                .await?;

            if seen.insert(employee_id) {
                employee_ids.push(employee_id);
            }
        }

        let now = Utc::now();
        let assignments = employee_ids
            .into_iter()
            .map(|employee_id| Assignment {
                id: AssignmentId(generate_uuid_v7()),
                organization_id: work_order.organization_id,
                work_order_id: work_order.id,
                employee_id,
                created_at: now,
            })
            .collect();

        Ok((assignments, created_employees))
    }

    async fn resolve_assignee(
        &mut self,
        work_order: &WorkOrder,
        assignee: crate::AssigneeRef,
        created_employees: &mut Vec<Employee>,
    ) -> Result<EmployeeId, CoreError> {
        match assignee {
            crate::AssigneeRef::Employee(employee_id) => {
                let employee = self
                    .employee_repository
                    .find_by_id(employee_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;

                if employee.organization_id != work_order.organization_id {
                    return Err(CoreError::NotFound);
                }

                Ok(employee_id)
            }
            crate::AssigneeRef::Member(user_id) => {
                self.member_repository
                    .find_by_org_and_user(work_order.organization_id, user_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;

                if let Some(existing) = self
                    .employee_repository
                    .find_by_user_id(work_order.organization_id, user_id)
                    .await?
                {
                    return Ok(existing.id);
                }

                let user = self
                    .user_repository
                    .find_by_id(user_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;

                let now = Utc::now();
                let created = self
                    .employee_repository
                    .insert(&Employee {
                        id: EmployeeId(generate_uuid_v7()),
                        organization_id: work_order.organization_id,
                        user_id: Some(user_id),
                        name: user.name,
                        // Never `Some(0)`: an on-the-fly record has no rate
                        // *yet*, which is not the same as "free".
                        hourly_rate_cents: None,
                        weekly_contract_minutes: 0,
                        deleted_at: None,
                        created_at: now,
                        updated_at: now,
                    })
                    .await?;

                let employee_id = created.id;
                created_employees.push(created);
                Ok(employee_id)
            }
        }
    }

    pub async fn soft_delete_work_order(&mut self, id: WorkOrderId) -> Result<(), CoreError> {
        self.get_work_order(id).await?;
        self.work_order_repository.soft_delete(id, Utc::now()).await
    }
}

fn validate_schedule(
    starts_at: chrono::DateTime<Utc>,
    ends_at: chrono::DateTime<Utc>,
) -> Result<(), CoreError> {
    if ends_at <= starts_at {
        return Err(CoreError::Conflict(
            "work order ends_at must be after starts_at".to_owned(),
        ));
    }

    Ok(())
}

fn validate_text_field(label: &str, value: &Option<String>) -> Result<(), CoreError> {
    if value.as_deref().is_some_and(|v| v.trim().is_empty()) {
        return Err(CoreError::Conflict(format!("{label} cannot be blank")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssigneeRef, CustomerContextId, CustomerId, OrganizationId, User, UserId, WorkOrderStatus,
        domain::{
            employee::ports::MockEmployeeRepository, member::ports::MockMemberRepository,
            user::ports::MockUserRepository, work_order::ports::MockWorkOrderRepository,
        },
    };
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn service(
        work_order_repository: MockWorkOrderRepository,
        employee_repository: MockEmployeeRepository,
        user_repository: MockUserRepository,
        member_repository: MockMemberRepository,
    ) -> WorkOrderService<
        MockWorkOrderRepository,
        MockEmployeeRepository,
        MockUserRepository,
        MockMemberRepository,
    > {
        WorkOrderService::new(
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        )
    }

    fn create_command() -> CreateWorkOrderCommand {
        let now = Utc::now();
        CreateWorkOrderCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            quote_id: None,
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            title: Some("Toiture".to_owned()),
            note: None,
        }
    }

    fn work_order(id: WorkOrderId, organization_id: OrganizationId) -> WorkOrder {
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
            status: WorkOrderStatus::Planned,
            title: Some("Toiture".to_owned()),
            note: None,
            assignments: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
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

    #[tokio::test]
    async fn create_work_order_persists_with_planned_status_and_no_assignments() {
        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_insert()
            .times(1)
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let created = service.create_work_order(create_command()).await.unwrap();

        assert_eq!(created.status, WorkOrderStatus::Planned);
        assert!(created.assignments.is_empty());
        assert_eq!(created.title.as_deref(), Some("Toiture"));
    }

    #[tokio::test]
    async fn create_work_order_rejects_ends_at_before_starts_at() {
        let mut service = service(
            MockWorkOrderRepository::new(),
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.ends_at = command.starts_at;

        let err = service.create_work_order(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_work_order_rejects_blank_title() {
        let mut service = service(
            MockWorkOrderRepository::new(),
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = create_command();
        command.title = Some("   ".to_owned());

        let err = service.create_work_order(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_work_order_returns_not_found_when_missing() {
        let id = WorkOrderId(Uuid::new_v4());
        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let err = service.get_work_order(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_work_orders_delegates_to_repo() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_list_by_organization()
            .with(eq(organization_id), eq(10), eq(20))
            .returning(move |org_id, _, _| {
                Box::pin(
                    async move { Ok((vec![work_order(WorkOrderId(Uuid::new_v4()), org_id)], 1)) },
                )
            });

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let (items, total) = service
            .list_work_orders(organization_id, 10, 20)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn patch_work_order_reschedules_without_touching_assignees() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);
        let new_starts_at = existing.starts_at + chrono::Duration::days(1);
        let new_ends_at = existing.ends_at + chrono::Duration::days(1);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_update()
            .withf(move |w| w.assignments.is_empty())
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.starts_at = Some(new_starts_at);
        command.ends_at = Some(new_ends_at);

        let (updated, created_employees) = service.patch_work_order(command).await.unwrap();

        assert_eq!(updated.starts_at, new_starts_at);
        assert_eq!(updated.ends_at, new_ends_at);
        assert!(created_employees.is_empty());
    }

    #[tokio::test]
    async fn patch_work_order_rejects_ends_at_before_merged_starts_at() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        // Force an invalid window: ends_at before the existing starts_at.
        let existing_starts_at = work_order(id, organization_id).starts_at;
        let mut command = PatchWorkOrderCommand::new(id);
        command.ends_at = Some(existing_starts_at - chrono::Duration::hours(1));

        let err = service.patch_work_order(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_work_order_can_clear_the_title() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_update()
            .withf(|w| w.title.is_none())
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.title = Some(None);

        let (updated, _) = service.patch_work_order(command).await.unwrap();

        assert!(updated.title.is_none());
    }

    #[tokio::test]
    async fn patch_work_order_assigns_an_existing_employee_in_the_same_org() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);
        let employee_id = EmployeeId(Uuid::new_v4());
        let target_employee = employee(employee_id, organization_id);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_update()
            .withf(move |w| w.assignments.len() == 1 && w.assignments[0].employee_id == employee_id)
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .with(eq(employee_id))
            .returning(move |_| {
                let e = target_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut service = service(
            work_order_repository,
            employee_repository,
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Employee(employee_id)]);

        let (updated, created_employees) = service.patch_work_order(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(updated.assignments[0].employee_id, employee_id);
        assert!(created_employees.is_empty());
    }

    #[tokio::test]
    async fn patch_work_order_rejects_an_employee_from_another_organization() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let other_org_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);
        let employee_id = EmployeeId(Uuid::new_v4());
        let foreign_employee = employee(employee_id, other_org_id);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .with(eq(employee_id))
            .returning(move |_| {
                let e = foreign_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut service = service(
            work_order_repository,
            employee_repository,
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Employee(employee_id)]);

        let err = service.patch_work_order(command).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn patch_work_order_creates_an_employee_for_a_member_without_one() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);
        let user_id = UserId(Uuid::new_v4());

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_update()
            .withf(|w| w.assignments.len() == 1)
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(organization_id), eq(user_id))
            .returning(move |org_id, user_id| {
                let m = crate::Member {
                    id: crate::MemberId(Uuid::new_v4()),
                    organization_id: org_id,
                    user_id,
                    joined_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_user_id()
            .with(eq(organization_id), eq(user_id))
            .returning(|_, _| Box::pin(async { Ok(None) }));
        employee_repository
            .expect_insert()
            .withf(move |e| {
                e.user_id == Some(user_id)
                    && e.hourly_rate_cents.is_none()
                    && e.weekly_contract_minutes == 0
                    && e.name == "Bob Member"
            })
            .times(1)
            .returning(|e| {
                let cloned = e.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(eq(user_id))
            .returning(move |id| {
                let now = Utc::now();
                let user = User {
                    id,
                    email: "bob@example.com".to_owned(),
                    username: "bob".to_owned(),
                    name: "Bob Member".to_owned(),
                    sub: "sub-bob".to_owned(),
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(user)) })
            });

        let mut service = service(
            work_order_repository,
            employee_repository,
            user_repository,
            member_repository,
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Member(user_id)]);

        let (updated, created_employees) = service.patch_work_order(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(created_employees.len(), 1);
        assert_eq!(created_employees[0].hourly_rate_cents, None);
        assert_eq!(created_employees[0].weekly_contract_minutes, 0);
    }

    #[tokio::test]
    async fn patch_work_order_reuses_an_employee_already_provisioned_for_the_member() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);
        let user_id = UserId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut existing_employee = employee(employee_id, organization_id);
        existing_employee.user_id = Some(user_id);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_update()
            .withf(move |w| w.assignments.len() == 1 && w.assignments[0].employee_id == employee_id)
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(organization_id), eq(user_id))
            .returning(move |org_id, user_id| {
                let m = crate::Member {
                    id: crate::MemberId(Uuid::new_v4()),
                    organization_id: org_id,
                    user_id,
                    joined_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_user_id()
            .with(eq(organization_id), eq(user_id))
            .returning(move |_, _| {
                let e = existing_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });
        // No `expect_insert`: reusing the existing record must not create a
        // second one for the same person.

        let mut service = service(
            work_order_repository,
            employee_repository,
            MockUserRepository::new(),
            member_repository,
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Member(user_id)]);

        let (updated, created_employees) = service.patch_work_order(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
        assert_eq!(updated.assignments[0].employee_id, employee_id);
        assert!(created_employees.is_empty());
    }

    #[tokio::test]
    async fn patch_work_order_rejects_a_member_outside_the_organization() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);
        let user_id = UserId(Uuid::new_v4());

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(organization_id), eq(user_id))
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            member_repository,
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.assignees = Some(vec![AssigneeRef::Member(user_id)]);

        let err = service.patch_work_order(command).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn patch_work_order_dedupes_repeated_assignees() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);
        let employee_id = EmployeeId(Uuid::new_v4());
        let target_employee = employee(employee_id, organization_id);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_update()
            .withf(|w| w.assignments.len() == 1)
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_find_by_id()
            .with(eq(employee_id))
            .returning(move |_| {
                let e = target_employee.clone();
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut service = service(
            work_order_repository,
            employee_repository,
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.assignees = Some(vec![
            AssigneeRef::Employee(employee_id),
            AssigneeRef::Employee(employee_id),
        ]);

        let (updated, _) = service.patch_work_order(command).await.unwrap();

        assert_eq!(updated.assignments.len(), 1);
    }

    #[tokio::test]
    async fn patch_work_order_can_clear_all_assignees() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut existing = work_order(id, organization_id);
        existing.assignments.push(Assignment {
            id: AssignmentId(Uuid::new_v4()),
            organization_id,
            work_order_id: id,
            employee_id: EmployeeId(Uuid::new_v4()),
            created_at: Utc::now(),
        });

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_update()
            .withf(|w| w.assignments.is_empty())
            .returning(|w| {
                let cloned = w.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        let mut command = PatchWorkOrderCommand::new(id);
        command.assignees = Some(Vec::new());

        let (updated, _) = service.patch_work_order(command).await.unwrap();

        assert!(updated.assignments.is_empty());
    }

    #[tokio::test]
    async fn soft_delete_work_order_checks_existence_then_deletes() {
        let id = WorkOrderId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = work_order(id, organization_id);

        let mut work_order_repository = MockWorkOrderRepository::new();
        work_order_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        work_order_repository
            .expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = service(
            work_order_repository,
            MockEmployeeRepository::new(),
            MockUserRepository::new(),
            MockMemberRepository::new(),
        );

        service.soft_delete_work_order(id).await.unwrap();
    }
}
