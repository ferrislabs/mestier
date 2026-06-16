use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Employee, EmployeeId, OrganizationId,
    domain::employee::{
        commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
        ports::EmployeeRepository,
    },
};

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

        let now = Utc::now();
        let employee = Employee {
            id: EmployeeId(generate_uuid_v7()),
            organization_id: command.organization_id,
            user_id: command.user_id,
            name: command.name,
            hourly_rate_cents: command.hourly_rate_cents,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&employee).await
    }

    pub async fn get_employee(&mut self, id: EmployeeId) -> Result<Employee, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
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

        let mut employee = self.get_employee(command.id).await?;
        employee.name = command.name;
        employee.hourly_rate_cents = command.hourly_rate_cents;
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

fn validate_rate(rate_cents: i32) -> Result<(), CoreError> {
    if rate_cents < 0 {
        return Err(CoreError::Conflict(
            "employee hourly rate cannot be negative".to_owned(),
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
            hourly_rate_cents: 3500,
            deleted_at: None,
            created_at: now,
            updated_at: now,
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
        let created = service
            .create_employee(CreateEmployeeCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                user_id: None,
                name: "Alice".to_owned(),
                hourly_rate_cents: 3500,
            })
            .await
            .unwrap();

        assert_eq!(created.name, "Alice");
        assert_eq!(created.hourly_rate_cents, 3500);
    }

    #[tokio::test]
    async fn create_employee_rejects_negative_rate() {
        let repo = MockEmployeeRepository::new();
        let mut service = EmployeeService::new(repo);

        let err = service
            .create_employee(CreateEmployeeCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                user_id: None,
                name: "Alice".to_owned(),
                hourly_rate_cents: -1,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
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
                hourly_rate_cents: 4200,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Bob");
        assert_eq!(updated.hourly_rate_cents, 4200);
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
