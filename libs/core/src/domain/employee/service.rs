use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Employee, EmployeeId, MemberId, OrganizationId,
    domain::employee::{
        commands::{RemoveEmployeeProfileCommand, UpsertEmployeeProfileCommand},
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

    /// Attaches a contractual profile to a member, or updates the one already
    /// attached. Idempotent by design: the caller states what the contract is,
    /// not whether a row exists.
    pub async fn upsert_employee_profile(
        &mut self,
        command: UpsertEmployeeProfileCommand,
    ) -> Result<Employee, CoreError> {
        validate_rate(command.hourly_rate_cents)?;
        validate_weekly_contract_minutes(command.weekly_contract_minutes)?;

        let now = Utc::now();

        match self.repo.find_by_member_id(command.member_id).await? {
            Some(mut existing) => {
                existing.hourly_rate_cents = command.hourly_rate_cents;
                existing.weekly_contract_minutes = command.weekly_contract_minutes;
                existing.updated_at = now;

                self.repo.update(&existing).await
            }
            None => {
                let employee = Employee {
                    id: EmployeeId(generate_uuid_v7()),
                    organization_id: command.organization_id,
                    member_id: command.member_id,
                    hourly_rate_cents: command.hourly_rate_cents,
                    weekly_contract_minutes: command.weekly_contract_minutes,
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };

                self.repo.insert(&employee).await
            }
        }
    }

    pub async fn get_employee(&mut self, id: EmployeeId) -> Result<Employee, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    /// The profile attached to a member, or [`CoreError::NotFound`] when the
    /// member has none — the read counterpart of the upsert, keyed the way
    /// callers address a person.
    pub async fn get_employee_by_member(
        &mut self,
        member_id: MemberId,
    ) -> Result<Employee, CoreError> {
        self.repo
            .find_by_member_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)
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

    /// Detaches the profile from a member. The seat and everything hanging off
    /// it — assignments, absences, work slots — survive: this removes a
    /// contract, not a person.
    pub async fn remove_employee_profile(
        &mut self,
        command: RemoveEmployeeProfileCommand,
    ) -> Result<(), CoreError> {
        let employee = self.get_employee_by_member(command.member_id).await?;
        self.repo.soft_delete(employee.id, Utc::now()).await
    }
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
    use crate::domain::employee::ports::MockEmployeeRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn employee(id: EmployeeId, member_id: MemberId) -> Employee {
        let now = Utc::now();
        Employee {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            member_id,
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn upsert_command(member_id: MemberId) -> UpsertEmployeeProfileCommand {
        UpsertEmployeeProfileCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            member_id,
            hourly_rate_cents: Some(3500),
            weekly_contract_minutes: 2100,
        }
    }

    #[tokio::test]
    async fn upsert_inserts_when_the_member_has_no_profile_yet() {
        let member_id = MemberId(Uuid::new_v4());
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id()
            .with(eq(member_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));
        repo.expect_insert().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let created = service
            .upsert_employee_profile(upsert_command(member_id))
            .await
            .unwrap();

        assert_eq!(created.member_id, member_id);
        assert_eq!(created.hourly_rate_cents, Some(3500));
        assert_eq!(created.weekly_contract_minutes, 2100);
    }

    /// The whole point of an upsert: a second call must not open a second
    /// profile for the same seat — `uq_employees_member_active` would reject it
    /// anyway, and turning a constraint violation into normal behaviour is not
    /// a design.
    #[tokio::test]
    async fn upsert_updates_the_profile_already_attached() {
        let member_id = MemberId(Uuid::new_v4());
        let existing = employee(EmployeeId(Uuid::new_v4()), member_id);
        let existing_id = existing.id;

        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id()
            .with(eq(member_id))
            .times(1)
            .returning(move |_| {
                let found = existing.clone();
                Box::pin(async move { Ok(Some(found)) })
            });
        repo.expect_insert().never();
        repo.expect_update().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let updated = service
            .upsert_employee_profile(UpsertEmployeeProfileCommand {
                hourly_rate_cents: Some(4200),
                ..upsert_command(member_id)
            })
            .await
            .unwrap();

        assert_eq!(updated.id, existing_id);
        assert_eq!(updated.hourly_rate_cents, Some(4200));
    }

    /// `None` is "not set", `Some(0)` is "genuinely free". Collapsing the two
    /// would feed a wrong cost into the profitability computation instead of
    /// refusing to produce one.
    #[tokio::test]
    async fn upsert_keeps_an_unset_rate_distinct_from_zero() {
        let member_id = MemberId(Uuid::new_v4());
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));
        repo.expect_insert().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let created = service
            .upsert_employee_profile(UpsertEmployeeProfileCommand {
                hourly_rate_cents: None,
                ..upsert_command(member_id)
            })
            .await
            .unwrap();

        assert_eq!(created.hourly_rate_cents, None);
    }

    #[tokio::test]
    async fn upsert_rejects_a_negative_rate() {
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id().never();
        repo.expect_insert().never();

        let mut service = EmployeeService::new(repo);
        let result = service
            .upsert_employee_profile(UpsertEmployeeProfileCommand {
                hourly_rate_cents: Some(-1),
                ..upsert_command(MemberId(Uuid::new_v4()))
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn upsert_rejects_a_contract_longer_than_a_week() {
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id().never();
        repo.expect_insert().never();

        let mut service = EmployeeService::new(repo);
        let result = service
            .upsert_employee_profile(UpsertEmployeeProfileCommand {
                weekly_contract_minutes: MINUTES_PER_WEEK + 1,
                ..upsert_command(MemberId(Uuid::new_v4()))
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn remove_profile_soft_deletes_the_profile_of_that_member() {
        let member_id = MemberId(Uuid::new_v4());
        let existing = employee(EmployeeId(Uuid::new_v4()), member_id);
        let existing_id = existing.id;

        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id()
            .with(eq(member_id))
            .times(1)
            .returning(move |_| {
                let found = existing.clone();
                Box::pin(async move { Ok(Some(found)) })
            });
        repo.expect_soft_delete()
            .withf(move |id, _| *id == existing_id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = EmployeeService::new(repo);

        assert!(
            service
                .remove_employee_profile(RemoveEmployeeProfileCommand { member_id })
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn remove_profile_reports_not_found_when_the_member_has_none() {
        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));
        repo.expect_soft_delete().never();

        let mut service = EmployeeService::new(repo);
        let result = service
            .remove_employee_profile(RemoveEmployeeProfileCommand {
                member_id: MemberId(Uuid::new_v4()),
            })
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
