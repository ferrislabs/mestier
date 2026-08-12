use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Employee, EmployeeId, MemberId, OrganizationId,
    application::{MestierUseCase, policy},
    domain::{
        employee::{
            commands::{RemoveEmployeeProfileCommand, UpsertEmployeeProfileCommand},
            service::EmployeeService,
        },
        member::ports::MemberRepository,
    },
};

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
    #[transactional(employee, member, role, authz)]
    pub async fn upsert_employee_profile(
        &self,
        actor: Subject,
        member_id: MemberId,
        hourly_rate_cents: Option<i32>,
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
        service
            .upsert_employee_profile(UpsertEmployeeProfileCommand {
                organization_id: member.organization_id,
                member_id,
                hourly_rate_cents,
                weekly_contract_minutes,
            })
            .await
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
}
