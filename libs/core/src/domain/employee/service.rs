use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Employee, EmployeeId, OrganizationId,
    domain::employee::{
        commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
        ports::EmployeeRepository,
    },
};

/// Upper bound of a weekly contractual base. A contract cannot span more time
/// than the week it is expressed in.
pub const MINUTES_PER_WEEK: i32 = 7 * 24 * 60;

pub struct EmployeeService<R>
where
    R: EmployeeRepository,
{
    repo: R,
}

impl<R> EmployeeService<R>
where
    R: EmployeeRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_employee(
        &mut self,
        command: CreateEmployeeCommand,
    ) -> Result<Employee, CoreError> {
        validate_name(&command.name)?;
        validate_rate(command.hourly_rate_cents)?;
        validate_weekly_contract_minutes(command.weekly_contract_minutes)?;

        let now = Utc::now();
        let employee = Employee {
            id: EmployeeId(generate_uuid_v7()),
            organization_id: command.organization_id,
            user_id: command.user_id,
            name: command.name,
            hourly_rate_cents: command.hourly_rate_cents,
            weekly_contract_minutes: command.weekly_contract_minutes,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&employee).await
    }

    pub async fn get_employee(&mut self, id: EmployeeId) -> Result<Employee, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn find_employee_by_user_id(
        &mut self,
        organization_id: OrganizationId,
        user_id: crate::UserId,
    ) -> Result<Option<Employee>, CoreError> {
        self.repo
            .find_by_user_id(organization_id, user_id)
            .await
    }

    pub async fn list_employees(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Employee>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn update_employee(
        &mut self,
        command: UpdateEmployeeCommand,
    ) -> Result<Employee, CoreError> {
        validate_name(&command.name)?;
        validate_rate(command.hourly_rate_cents)?;
        validate_weekly_contract_minutes(command.weekly_contract_minutes)?;

        let mut employee = self.get_employee(command.id).await?;
        employee.name = command.name;
        employee.hourly_rate_cents = command.hourly_rate_cents;
        employee.weekly_contract_minutes = command.weekly_contract_minutes;
        employee.updated_at = Utc::now();

        self.repo.update(&employee).await
    }

    pub async fn link_employee_user(
        &mut self,
        command: LinkEmployeeUserCommand,
    ) -> Result<Employee, CoreError> {
        let mut employee = self.get_employee(command.id).await?;
        employee.user_id = command.user_id;
        employee.updated_at = Utc::now();

        self.repo.update(&employee).await
    }

    pub async fn soft_delete_employee(&mut self, id: EmployeeId) -> Result<(), CoreError> {
        self.get_employee(id).await?;
        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "employee name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn validate_rate(rate_cents: Option<i32>) -> Result<(), CoreError> {
    if rate_cents.is_some_and(|cents| cents < 0) {
        return Err(CoreError::Conflict(
            "employee hourly rate cannot be negative".to_owned(),
        ));
    }

    Ok(())
}

fn validate_weekly_contract_minutes(minutes: i32) -> Result<(), CoreError> {
    if minutes < 0 {
        return Err(CoreError::Conflict(
            "employee weekly contract cannot be negative".to_owned(),
        ));
    }

    if minutes > MINUTES_PER_WEEK {
        return Err(CoreError::Conflict(
            "employee weekly contract cannot exceed a week".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UserId, domain::employee::ports::MockEmployeeRepository};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn employee(id: EmployeeId) -> Employee {
        let now = Utc::now();
        Employee {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            user_id: None,
            name: "Alice".to_owned(),
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn create_command() -> CreateEmployeeCommand {
        CreateEmployeeCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            user_id: None,
            name: "Alice".to_owned(),
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
        }
    }

    #[tokio::test]
    async fn create_employee_persists_via_repo() {
        let mut repo = MockEmployeeRepository::new();
        repo.expect_insert().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let created = service.create_employee(create_command()).await.unwrap();

        assert_eq!(created.name, "Alice");
        assert_eq!(created.hourly_rate_cents, Some(3500));
        assert_eq!(created.weekly_contract_minutes, 2100);
    }

    #[tokio::test]
    async fn create_employee_accepts_an_unset_hourly_rate() {
        let mut repo = MockEmployeeRepository::new();
        repo.expect_insert().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let created = service
            .create_employee(CreateEmployeeCommand {
                hourly_rate_cents: None,
                ..create_command()
            })
            .await
            .unwrap();

        // `None` means "rate not set"; `Some(0)` would mean "genuinely free".
        assert_eq!(created.hourly_rate_cents, None);
    }

    #[tokio::test]
    async fn create_employee_rejects_negative_rate() {
        let repo = MockEmployeeRepository::new();
        let mut service = EmployeeService::new(repo);

        let err = service
            .create_employee(CreateEmployeeCommand {
                hourly_rate_cents: Some(-1),
                ..create_command()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_employee_rejects_negative_weekly_contract_minutes() {
        let repo = MockEmployeeRepository::new();
        let mut service = EmployeeService::new(repo);

        let err = service
            .create_employee(CreateEmployeeCommand {
                weekly_contract_minutes: -1,
                ..create_command()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_employee_rejects_a_contract_longer_than_a_week() {
        let repo = MockEmployeeRepository::new();
        let mut service = EmployeeService::new(repo);

        let err = service
            .create_employee(CreateEmployeeCommand {
                weekly_contract_minutes: MINUTES_PER_WEEK + 1,
                ..create_command()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_employee_accepts_a_full_week_contract() {
        let mut repo = MockEmployeeRepository::new();
        repo.expect_insert().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let created = service
            .create_employee(CreateEmployeeCommand {
                weekly_contract_minutes: MINUTES_PER_WEEK,
                ..create_command()
            })
            .await
            .unwrap();

        assert_eq!(created.weekly_contract_minutes, MINUTES_PER_WEEK);
    }

    #[tokio::test]
    async fn update_employee_mutates_existing_employee() {
        let id = EmployeeId(Uuid::new_v4());
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(employee(id))) }));
        repo.expect_update().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let updated = service
            .update_employee(UpdateEmployeeCommand {
                id,
                name: "Bob".to_owned(),
                hourly_rate_cents: Some(4200),
                weekly_contract_minutes: 1920,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Bob");
        assert_eq!(updated.hourly_rate_cents, Some(4200));
        assert_eq!(updated.weekly_contract_minutes, 1920);
    }

    #[tokio::test]
    async fn update_employee_can_clear_the_hourly_rate() {
        let id = EmployeeId(Uuid::new_v4());
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(employee(id))) }));
        repo.expect_update().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let updated = service
            .update_employee(UpdateEmployeeCommand {
                id,
                name: "Bob".to_owned(),
                hourly_rate_cents: None,
                weekly_contract_minutes: 2100,
            })
            .await
            .unwrap();

        assert_eq!(updated.hourly_rate_cents, None);
    }

    #[tokio::test]
    async fn link_employee_user_sets_nullable_user_id() {
        let id = EmployeeId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(employee(id))) }));
        repo.expect_update().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let updated = service
            .link_employee_user(LinkEmployeeUserCommand {
                id,
                user_id: Some(user_id),
            })
            .await
            .unwrap();

        assert_eq!(updated.user_id, Some(user_id));
    }

    #[tokio::test]
    async fn list_employees_delegates_to_repo() {
        let org_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockEmployeeRepository::new();
        repo.expect_list_by_organization()
            .with(eq(org_id), eq(20), eq(40))
            .times(1)
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![employee(EmployeeId(Uuid::new_v4()))], 1)) })
            });

        let mut service = EmployeeService::new(repo);
        let (items, total) = service.list_employees(org_id, 20, 40).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn soft_delete_employee_checks_existence_then_deletes() {
        let id = EmployeeId(Uuid::new_v4());
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(employee(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = EmployeeService::new(repo);

        service.soft_delete_employee(id).await.unwrap();
    }
}
