use authz::{Resource, Subject};
use chrono::{NaiveDate, Utc};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Employee, EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, MemberId, OrganizationId,
    application::{MestierUseCase, policy},
    domain::{
        employee::{
            commands::{
                CorrectEmployeeCostBasisCommand, RemoveEmployeeProfileCommand,
                SetEmployeeCostBasisCommand, UpsertEmployeeProfileCommand,
            },
            ports::EmployeeRepository,
            service::{EmployeeCostBasisService, EmployeeService},
        },
        member::ports::MemberRepository,
    },
};

mod tests;

impl MestierUseCase {
    /// Attaches a contractual profile to a member, or updates the one already
    /// there.
    ///
    /// The seat is loaded first and authorization runs against *its*
    /// organization, never one taken from the request path — the same rule the
    /// member item routes follow, and the reason a bare `/members/{id}/...`
    /// route cannot be turned into a cross-tenant IDOR.
    ///
    /// Composition lives here rather than inside `EmployeeService`: the
    /// profile and the seat are separate aggregates, and the application seam
    /// is where this repository joins them (as `patch_task` does for task
    /// labels).
    ///
    /// Also versions the change: every call dates a new (or edited) cost
    /// basis version as of today, so a rate entered through this same
    /// endpoint the epic has always had is still costed correctly by
    /// profitability — "from today" is what somebody calling this without a
    /// date means. `employees`' own cost columns stay in sync as a
    /// projection of that version, per the versioned-history design (see
    /// `employee_cost_bases`).
    #[transactional(employee, employee_cost_basis, member, role, authz)]
    pub async fn upsert_employee_profile(
        &self,
        actor: Subject,
        member_id: MemberId,
        hourly_rate_cents: Option<i32>,
        is_salaried: bool,
        monthly_cost_cents: Option<i32>,
        weekly_contract_minutes: i32,
    ) -> Result<Employee, CoreError> {
        // `#[transactional]` hands the repositories over immutably; the
        // policy engine needs them mutable to walk roles.
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let member = member_repository
            .find_by_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            member.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "member.manage",
            Resource::new("member", member.id.0.to_string()),
        )
        .await?;

        let mut service = EmployeeService::new(employee_repository);
        let employee = service
            .upsert_employee_profile(UpsertEmployeeProfileCommand {
                organization_id: member.organization_id,
                member_id,
                hourly_rate_cents,
                is_salaried,
                monthly_cost_cents,
                weekly_contract_minutes,
            })
            .await?;

        let mut cost_basis_service = EmployeeCostBasisService::new(employee_cost_basis_repository);
        cost_basis_service
            .set_cost_basis(SetEmployeeCostBasisCommand {
                organization_id: employee.organization_id,
                employee_id: employee.id,
                effective_from: Utc::now().date_naive(),
                is_salaried: employee.is_salaried,
                hourly_rate_cents: employee.hourly_rate_cents,
                monthly_cost_cents: employee.monthly_cost_cents,
                weekly_contract_minutes: employee.weekly_contract_minutes,
            })
            .await?;

        Ok(employee)
    }

    /// Detaches the contractual profile from a member. The seat and its
    /// history — assignments, absences, work slots — survive.
    #[transactional(employee, member, role, authz)]
    pub async fn remove_employee_profile(
        &self,
        actor: Subject,
        member_id: MemberId,
    ) -> Result<(), CoreError> {
        // `#[transactional]` hands the repositories over immutably; the
        // policy engine needs them mutable to walk roles.
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let member = member_repository
            .find_by_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            member.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "member.manage",
            Resource::new("member", member.id.0.to_string()),
        )
        .await?;

        let mut service = EmployeeService::new(employee_repository);
        service
            .remove_employee_profile(RemoveEmployeeProfileCommand { member_id })
            .await
    }

    #[transactional(employee)]
    pub async fn get_employee(&self, id: EmployeeId) -> Result<Employee, CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.get_employee(id).await
    }

    /// The profile attached to a member, or `CoreError::NotFound` when there
    /// is none — a member without a contract is normal, not an error the
    /// caller should treat as a failure.
    #[transactional(employee)]
    pub async fn get_employee_by_member(&self, member_id: MemberId) -> Result<Employee, CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.get_employee_by_member(member_id).await
    }

    /// Rate/contract data is sensitive (see #182's note on `hourly_rate_cents`
    /// having no read filter anywhere yet), so unlike the other reference-data
    /// lists (equipment, products, service rates), this one gates on
    /// `member.manage` rather than plain organization membership.
    #[transactional(employee, member, role, authz)]
    pub async fn list_employees(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Employee>, u64), CoreError> {
        // `#[transactional]` hands the repositories over immutably; the
        // policy engine needs them mutable to walk roles.
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "member.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await?;

        let mut service = EmployeeService::new(employee_repository);
        service.list_employees(organization_id, limit, offset).await
    }

    /// The whole cost history of one employee, oldest first. Same gate as
    /// [`Self::list_employees`]: this is the same sensitive figure, just
    /// versioned.
    #[transactional(employee, employee_cost_basis, member, role, authz)]
    pub async fn list_employee_cost_bases(
        &self,
        actor: Subject,
        employee_id: EmployeeId,
    ) -> Result<Vec<EmployeeCostBasis>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut employee_repository = employee_repository;

        let employee = employee_repository
            .find_by_id(employee_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            employee.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "member.manage",
            Resource::new("member", employee.member_id.0.to_string()),
        )
        .await?;

        let mut service = EmployeeCostBasisService::new(employee_cost_basis_repository);
        service.list_cost_bases(employee_id).await
    }

    /// Dates a cost basis change: closes the open version at `effective_from`
    /// and opens a new one, or edits the open version in place when
    /// `effective_from` matches it exactly. The standalone counterpart of the
    /// dating [`Self::upsert_employee_profile`] does implicitly for "today" —
    /// this is the one that lets a caller say "from this date" instead.
    #[allow(clippy::too_many_arguments)]
    #[transactional(employee, employee_cost_basis, member, role, authz)]
    pub async fn set_employee_cost_basis(
        &self,
        actor: Subject,
        employee_id: EmployeeId,
        effective_from: NaiveDate,
        is_salaried: bool,
        hourly_rate_cents: Option<i32>,
        monthly_cost_cents: Option<i32>,
        weekly_contract_minutes: i32,
    ) -> Result<EmployeeCostBasis, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut employee_repository = employee_repository;

        let employee = employee_repository
            .find_by_id(employee_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            employee.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "member.manage",
            Resource::new("member", employee.member_id.0.to_string()),
        )
        .await?;

        let mut service = EmployeeCostBasisService::new(employee_cost_basis_repository);
        service
            .set_cost_basis(SetEmployeeCostBasisCommand {
                organization_id: employee.organization_id,
                employee_id,
                effective_from,
                is_salaried,
                hourly_rate_cents,
                monthly_cost_cents,
                weekly_contract_minutes,
            })
            .await
    }

    /// Corrects a cost basis version that was entered wrong — the dangerous
    /// verb, because it rewrites history on purpose rather than dating a new
    /// change.
    ///
    /// The version is loaded first and authorization runs against *its own*
    /// `organization_id`, never one taken from the request path: a bare
    /// `/cost-bases/{id}` route has no organization to trust otherwise, the
    /// same reason `upsert_employee_profile` loads the seat before
    /// authorizing.
    #[allow(clippy::too_many_arguments)]
    #[transactional(employee_cost_basis, member, role, authz)]
    pub async fn correct_employee_cost_basis(
        &self,
        actor: Subject,
        id: EmployeeCostBasisId,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        is_salaried: bool,
        hourly_rate_cents: Option<i32>,
        monthly_cost_cents: Option<i32>,
        weekly_contract_minutes: i32,
    ) -> Result<EmployeeCostBasis, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = EmployeeCostBasisService::new(employee_cost_basis_repository);
        let current = service.get_cost_basis(id).await?;

        let actor = policy::enrich_for_organization(
            actor,
            current.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "member.manage",
            Resource::new("member", current.employee_id.0.to_string()),
        )
        .await?;

        service
            .correct_cost_basis(CorrectEmployeeCostBasisCommand {
                id,
                effective_from,
                effective_to,
                is_salaried,
                hourly_rate_cents,
                monthly_cost_cents,
                weekly_contract_minutes,
            })
            .await
    }
}
