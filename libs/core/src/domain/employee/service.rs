use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Employee, EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, MemberId, OrganizationId,
    domain::employee::{
        commands::{
            CorrectEmployeeCostBasisCommand, RemoveEmployeeProfileCommand,
            SetEmployeeCostBasisCommand, UpsertEmployeeProfileCommand,
        },
        ports::{EmployeeCostBasisRepository, EmployeeRepository},
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
        validate_monthly_cost(command.monthly_cost_cents)?;
        validate_weekly_contract_minutes(command.weekly_contract_minutes)?;

        // The two cost bases are exclusive, and the unused one is cleared rather
        // than merely ignored downstream: a figure that lingers unused reads as
        // forgotten-to-update the next time somebody looks at the row. The
        // `chk_employees_one_cost_basis` constraint enforces the same thing one
        // layer down.
        let (hourly_rate_cents, monthly_cost_cents) = if command.is_salaried {
            (None, command.monthly_cost_cents)
        } else {
            (command.hourly_rate_cents, None)
        };

        let now = Utc::now();

        match self.repo.find_by_member_id(command.member_id).await? {
            Some(mut existing) => {
                existing.hourly_rate_cents = hourly_rate_cents;
                existing.is_salaried = command.is_salaried;
                existing.monthly_cost_cents = monthly_cost_cents;
                existing.weekly_contract_minutes = command.weekly_contract_minutes;
                existing.updated_at = now;

                self.repo.update(&existing).await
            }
            None => {
                let employee = Employee {
                    id: EmployeeId(generate_uuid_v7()),
                    organization_id: command.organization_id,
                    member_id: command.member_id,
                    hourly_rate_cents,
                    is_salaried: command.is_salaried,
                    monthly_cost_cents,
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

/// Versions what an employee costs. Kept separate from [`EmployeeService`]
/// rather than folded into it: the two repositories serve different
/// concerns (the contractual profile itself vs. its dated history), and
/// composing them is the application layer's job — the same reasoning
/// `upsert_employee_profile`'s use case gives for joining the profile and
/// the seat, and the reason [`WorkTimeService`](crate::domain::work_time::service::WorkTimeService)
/// merges its two repositories only because both serve the very same
/// `get_work_time` read.
pub struct EmployeeCostBasisService<C>
where
    C: EmployeeCostBasisRepository,
{
    repo: C,
}

impl<C> EmployeeCostBasisService<C>
where
    C: EmployeeCostBasisRepository,
{
    pub fn new(repo: C) -> Self {
        Self { repo }
    }

    /// Sets what `command.employee_id` costs from `command.effective_from`
    /// onward.
    ///
    /// - No open version exists yet: the command becomes the employee's
    ///   first cost basis version.
    /// - An open version exists and shares `command.effective_from`: the
    ///   same version is being corrected the same day — its fields are
    ///   replaced in place, so calling this twice running never accumulates
    ///   a second row.
    /// - An open version exists and `command.effective_from` is later: a
    ///   genuinely new version starts, so the open one is closed
    ///   (`effective_to` set to the new version's start) rather than
    ///   overwritten — the old version stays in the database as history.
    /// - An open version exists and `command.effective_from` is earlier:
    ///   rejected — refusing is what keeps history from being silently
    ///   reordered.
    pub async fn set_cost_basis(
        &mut self,
        command: SetEmployeeCostBasisCommand,
    ) -> Result<EmployeeCostBasis, CoreError> {
        validate_rate(command.hourly_rate_cents)?;
        validate_monthly_cost(command.monthly_cost_cents)?;
        validate_weekly_contract_minutes(command.weekly_contract_minutes)?;

        // The two cost bases are exclusive, same rule as
        // `EmployeeService::upsert_employee_profile`.
        let (hourly_rate_cents, monthly_cost_cents) = if command.is_salaried {
            (None, command.monthly_cost_cents)
        } else {
            (command.hourly_rate_cents, None)
        };

        let now = Utc::now();
        match self.repo.find_open_by_employee(command.employee_id).await? {
            None => {
                self.repo
                    .insert(&new_basis(
                        &command,
                        hourly_rate_cents,
                        monthly_cost_cents,
                        now,
                    ))
                    .await
            }
            Some(mut open) if open.effective_from == command.effective_from => {
                open.is_salaried = command.is_salaried;
                open.hourly_rate_cents = hourly_rate_cents;
                open.monthly_cost_cents = monthly_cost_cents;
                open.weekly_contract_minutes = command.weekly_contract_minutes;
                open.updated_at = now;
                self.repo.update(&open).await
            }
            Some(open) if command.effective_from > open.effective_from => {
                self.repo
                    .set_effective_to(open.id, command.effective_from)
                    .await?;
                self.repo
                    .insert(&new_basis(
                        &command,
                        hourly_rate_cents,
                        monthly_cost_cents,
                        now,
                    ))
                    .await
            }
            Some(_) => Err(CoreError::Conflict(
                "cannot set a cost basis version before the one currently in effect".to_owned(),
            )),
        }
    }

    /// The whole history of `employee_id`, oldest first.
    pub async fn list_cost_bases(
        &mut self,
        employee_id: EmployeeId,
    ) -> Result<Vec<EmployeeCostBasis>, CoreError> {
        self.repo.list_by_employee(employee_id).await
    }

    /// One version by its own id, or [`CoreError::NotFound`]. The read half
    /// of the "bare id derives its organization from the loaded row"
    /// pattern: a caller keyed on a cost basis id alone loads the row here
    /// before it can know which organization to authorize against.
    pub async fn get_cost_basis(
        &mut self,
        id: EmployeeCostBasisId,
    ) -> Result<EmployeeCostBasis, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    /// Corrects a version that was entered wrong — the dangerous verb. Unlike
    /// [`Self::set_cost_basis`] this never opens a new version or closes
    /// another one: it rewrites the named version in place, dates included,
    /// and the database's exclusion constraint is what refuses a correction
    /// that would make two versions overlap.
    pub async fn correct_cost_basis(
        &mut self,
        command: CorrectEmployeeCostBasisCommand,
    ) -> Result<EmployeeCostBasis, CoreError> {
        validate_rate(command.hourly_rate_cents)?;
        validate_monthly_cost(command.monthly_cost_cents)?;
        validate_weekly_contract_minutes(command.weekly_contract_minutes)?;
        if let Some(effective_to) = command.effective_to
            && effective_to <= command.effective_from
        {
            return Err(CoreError::Conflict(
                "cost basis effective_to must be after effective_from".to_owned(),
            ));
        }

        let mut basis = self.get_cost_basis(command.id).await?;

        let (hourly_rate_cents, monthly_cost_cents) = if command.is_salaried {
            (None, command.monthly_cost_cents)
        } else {
            (command.hourly_rate_cents, None)
        };

        basis.effective_from = command.effective_from;
        basis.effective_to = command.effective_to;
        basis.is_salaried = command.is_salaried;
        basis.hourly_rate_cents = hourly_rate_cents;
        basis.monthly_cost_cents = monthly_cost_cents;
        basis.weekly_contract_minutes = command.weekly_contract_minutes;
        basis.updated_at = Utc::now();

        self.repo.update(&basis).await
    }
}

fn new_basis(
    command: &SetEmployeeCostBasisCommand,
    hourly_rate_cents: Option<i32>,
    monthly_cost_cents: Option<i32>,
    now: chrono::DateTime<Utc>,
) -> EmployeeCostBasis {
    EmployeeCostBasis {
        id: EmployeeCostBasisId(generate_uuid_v7()),
        organization_id: command.organization_id,
        employee_id: command.employee_id,
        effective_from: command.effective_from,
        effective_to: None,
        is_salaried: command.is_salaried,
        hourly_rate_cents,
        monthly_cost_cents,
        weekly_contract_minutes: command.weekly_contract_minutes,
        created_at: now,
        updated_at: now,
    }
}

/// Weeks in a year over months in a year: the factor that turns a weekly
/// contract into a monthly one. 35 h a week is 151,67 h a month, not 140.
const WEEKS_PER_MONTH_NUMERATOR: i64 = 52;
const WEEKS_PER_MONTH_DENOMINATOR: i64 = 12;

/// What an hour of a salaried person costs, or `None` when it cannot be said.
///
/// `None` for a missing amount, and equally for a contract of zero hours: a
/// salary spread over no contracted time has no hourly equivalent, and inventing
/// one would be worse than admitting the gap. Profitability treats both the same
/// way it treats an unset hourly rate — it refuses to cost the time rather than
/// counting it as free.
///
/// Derived on every read rather than stored, so a contract change cannot leave a
/// stale rate behind.
pub fn salaried_hourly_rate_cents(
    monthly_cost_cents: Option<i32>,
    weekly_contract_minutes: i32,
) -> Option<i32> {
    let monthly_cost_cents = i64::from(monthly_cost_cents?);
    if weekly_contract_minutes <= 0 {
        return None;
    }

    // Monthly contracted minutes, kept in minutes so the only division is the
    // last one: an intermediate "hours per month" would round twice.
    let monthly_minutes = i64::from(weekly_contract_minutes) * WEEKS_PER_MONTH_NUMERATOR
        / WEEKS_PER_MONTH_DENOMINATOR;
    if monthly_minutes <= 0 {
        return None;
    }

    let hourly = (monthly_cost_cents * 60 + monthly_minutes / 2) / monthly_minutes;

    i32::try_from(hourly).ok()
}

fn validate_monthly_cost(monthly_cost_cents: Option<i32>) -> Result<(), CoreError> {
    if monthly_cost_cents.is_some_and(|cents| cents < 0) {
        return Err(CoreError::Conflict(
            "employee monthly cost cannot be negative".to_owned(),
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
    use crate::domain::employee::ports::{MockEmployeeCostBasisRepository, MockEmployeeRepository};
    use chrono::NaiveDate;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn employee(id: EmployeeId, member_id: MemberId) -> Employee {
        let now = Utc::now();
        Employee {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            member_id,
            hourly_rate_cents: Some(3500),
            is_salaried: false,
            monthly_cost_cents: None,
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
            is_salaried: false,
            monthly_cost_cents: None,
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

    /// A salaried person has no meaningful hourly figure, so a rate provided
    /// alongside `is_salaried` is not stored — the two would otherwise say
    /// different things about the same person.
    #[tokio::test]
    async fn upsert_clears_the_rate_when_the_employee_is_salaried() {
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
                hourly_rate_cents: Some(3500),
                is_salaried: true,
                monthly_cost_cents: None,
                ..upsert_command(member_id)
            })
            .await
            .unwrap();

        assert_eq!(created.hourly_rate_cents, None);
        assert!(created.is_salaried);
    }

    /// Turning an hourly profile salaried must drop the rate it already had,
    /// not just refuse a new one.
    #[tokio::test]
    async fn upsert_clears_an_existing_rate_when_turned_salaried() {
        let member_id = MemberId(Uuid::new_v4());
        let existing = employee(EmployeeId(Uuid::new_v4()), member_id);

        let mut repo = MockEmployeeRepository::new();
        repo.expect_find_by_member_id()
            .times(1)
            .returning(move |_| {
                let found = existing.clone();
                Box::pin(async move { Ok(Some(found)) })
            });
        repo.expect_update().times(1).returning(|e| {
            let employee = e.clone();
            Box::pin(async move { Ok(employee) })
        });

        let mut service = EmployeeService::new(repo);
        let updated = service
            .upsert_employee_profile(UpsertEmployeeProfileCommand {
                is_salaried: true,
                monthly_cost_cents: None,
                ..upsert_command(member_id)
            })
            .await
            .unwrap();

        assert_eq!(updated.hourly_rate_cents, None);
        assert!(updated.is_salaried);
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

    /// 3 500 € a month on a 35 h contract: 151,67 h, so 23,08 € an hour. The
    /// figure a foreman can check on a calculator.
    #[test]
    fn a_full_time_salary_becomes_its_hourly_equivalent() {
        assert_eq!(
            salaried_hourly_rate_cents(Some(350_000), 2_100),
            Some(2_308)
        );
    }

    /// Same salary, half the contract: an hour of that person costs about twice
    /// as much. This is why the divisor is the contract and not a flat 151,67.
    #[test]
    fn a_half_time_contract_doubles_the_hourly_cost() {
        assert_eq!(
            salaried_hourly_rate_cents(Some(350_000), 1_050),
            Some(4_615)
        );
    }

    #[test]
    fn no_amount_means_no_hourly_equivalent() {
        assert_eq!(salaried_hourly_rate_cents(None, 2_100), None);
    }

    /// A salary spread over no contracted time has no hourly equivalent, and
    /// inventing one would be worse than admitting the gap.
    #[test]
    fn no_contracted_time_means_no_hourly_equivalent() {
        assert_eq!(salaried_hourly_rate_cents(Some(350_000), 0), None);
        assert_eq!(salaried_hourly_rate_cents(Some(350_000), -1), None);
    }

    /// `Some(0)` is a real answer: an intern paid nothing costs nothing, which
    /// is different from a salary nobody entered.
    #[test]
    fn a_salary_of_zero_costs_zero_rather_than_nothing_known() {
        assert_eq!(salaried_hourly_rate_cents(Some(0), 2_100), Some(0));
    }

    // -- EmployeeCostBasisService::set_cost_basis --------------------------

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn cost_basis_command(
        employee_id: EmployeeId,
        effective_from: NaiveDate,
    ) -> SetEmployeeCostBasisCommand {
        SetEmployeeCostBasisCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            effective_from,
            is_salaried: false,
            hourly_rate_cents: Some(3500),
            monthly_cost_cents: None,
            weekly_contract_minutes: 2100,
        }
    }

    fn cost_basis(
        employee_id: EmployeeId,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
    ) -> EmployeeCostBasis {
        let now = Utc::now();
        EmployeeCostBasis {
            id: EmployeeCostBasisId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            effective_from,
            effective_to,
            is_salaried: false,
            hourly_rate_cents: Some(3000),
            monthly_cost_cents: None,
            weekly_contract_minutes: 2100,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn set_cost_basis_inserts_a_first_version_when_none_is_open() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut repo = MockEmployeeCostBasisRepository::new();
        repo.expect_find_open_by_employee()
            .with(eq(employee_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));
        repo.expect_insert().times(1).returning(|b| {
            let basis = b.clone();
            Box::pin(async move { Ok(basis) })
        });

        let mut service = EmployeeCostBasisService::new(repo);
        let created = service
            .set_cost_basis(cost_basis_command(employee_id, date(2026, 1, 1)))
            .await
            .unwrap();

        assert_eq!(created.employee_id, employee_id);
        assert_eq!(created.effective_from, date(2026, 1, 1));
        assert_eq!(created.effective_to, None);
        assert_eq!(created.hourly_rate_cents, Some(3500));
    }

    /// The whole point of dating a change: calling it twice with the same
    /// `effective_from` must edit the same row, never accumulate a second
    /// version — `uq_employee_cost_bases_open_version` would reject it
    /// anyway, and turning a constraint violation into normal behaviour is
    /// not a design.
    #[tokio::test]
    async fn set_cost_basis_with_the_same_effective_from_replaces_in_place() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing_id = EmployeeCostBasisId(Uuid::new_v4());
        let mut existing = cost_basis(employee_id, date(2026, 1, 1), None);
        existing.id = existing_id;

        let mut repo = MockEmployeeCostBasisRepository::new();
        repo.expect_find_open_by_employee()
            .with(eq(employee_id))
            .times(1)
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        repo.expect_update()
            .withf(move |b| b.id == existing_id && b.hourly_rate_cents == Some(3500))
            .times(1)
            .returning(|b| {
                let basis = b.clone();
                Box::pin(async move { Ok(basis) })
            });
        // No `expect_insert`: the same version must be edited in place, not
        // duplicated.

        let mut service = EmployeeCostBasisService::new(repo);
        let updated = service
            .set_cost_basis(cost_basis_command(employee_id, date(2026, 1, 1)))
            .await
            .unwrap();

        assert_eq!(updated.id, existing_id);
        assert_eq!(updated.hourly_rate_cents, Some(3500));
    }

    #[tokio::test]
    async fn set_cost_basis_with_a_later_effective_from_closes_the_open_version_and_opens_a_new_one()
     {
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing_id = EmployeeCostBasisId(Uuid::new_v4());
        let mut existing = cost_basis(employee_id, date(2026, 1, 1), None);
        existing.id = existing_id;

        let mut repo = MockEmployeeCostBasisRepository::new();
        repo.expect_find_open_by_employee()
            .with(eq(employee_id))
            .times(1)
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        repo.expect_set_effective_to()
            .withf(move |id, effective_to| *id == existing_id && *effective_to == date(2026, 6, 1))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        repo.expect_insert()
            .withf(|b| b.effective_from == date(2026, 6, 1))
            .times(1)
            .returning(|b| {
                let basis = b.clone();
                Box::pin(async move { Ok(basis) })
            });

        let mut service = EmployeeCostBasisService::new(repo);
        let created = service
            .set_cost_basis(cost_basis_command(employee_id, date(2026, 6, 1)))
            .await
            .unwrap();

        assert_eq!(created.effective_from, date(2026, 6, 1));
        assert_eq!(created.effective_to, None);
    }

    #[tokio::test]
    async fn set_cost_basis_rejects_an_effective_from_before_the_open_version() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing = cost_basis(employee_id, date(2026, 6, 1), None);

        let mut repo = MockEmployeeCostBasisRepository::new();
        repo.expect_find_open_by_employee()
            .with(eq(employee_id))
            .times(1)
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        repo.expect_insert().never();
        repo.expect_update().never();
        repo.expect_set_effective_to().never();

        let mut service = EmployeeCostBasisService::new(repo);
        let err = service
            .set_cost_basis(cost_basis_command(employee_id, date(2026, 1, 1)))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    /// A salaried version has no meaningful hourly figure, mirroring
    /// `EmployeeService::upsert_employee_profile`'s own rule.
    #[tokio::test]
    async fn set_cost_basis_clears_the_rate_when_the_version_is_salaried() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut repo = MockEmployeeCostBasisRepository::new();
        repo.expect_find_open_by_employee()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));
        repo.expect_insert().times(1).returning(|b| {
            let basis = b.clone();
            Box::pin(async move { Ok(basis) })
        });

        let mut service = EmployeeCostBasisService::new(repo);
        let created = service
            .set_cost_basis(SetEmployeeCostBasisCommand {
                is_salaried: true,
                hourly_rate_cents: Some(3500),
                monthly_cost_cents: Some(350_000),
                ..cost_basis_command(employee_id, date(2026, 1, 1))
            })
            .await
            .unwrap();

        assert_eq!(created.hourly_rate_cents, None);
        assert_eq!(created.monthly_cost_cents, Some(350_000));
        assert!(created.is_salaried);
    }

    #[tokio::test]
    async fn set_cost_basis_rejects_a_negative_rate() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut repo = MockEmployeeCostBasisRepository::new();
        repo.expect_find_open_by_employee().never();
        repo.expect_insert().never();

        let mut service = EmployeeCostBasisService::new(repo);
        let err = service
            .set_cost_basis(SetEmployeeCostBasisCommand {
                hourly_rate_cents: Some(-1),
                ..cost_basis_command(employee_id, date(2026, 1, 1))
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }
}
