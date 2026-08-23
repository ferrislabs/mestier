use common::CoreError;
use mestier_macros::transactional;

use crate::{
    UserId,
    application::MestierUseCase,
    domain::organization::{
        Organization, OrganizationId,
        commands::{
            CreateOrganizationCommand, UpdateLegalIdentityCommand, UpdateOrganizationCommand,
        },
        service::OrganizationService,
    },
};

impl MestierUseCase {
    /// Also seeds the organization's three preset task labels — see
    /// `OrganizationService::create_organization`. `task_label` is added
    /// only to this one method's repository list; the other use cases below
    /// stay unaware of the label aggregate.
    #[transactional(organization, role, member, user, authz, task_label)]
    pub async fn create_organization(
        &self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service
            .create_organization(command, task_label_repository)
            .await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn get_organization(&self, id: OrganizationId) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.get_organization(id).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn list_organizations_for_user(
        &self,
        sub: &str,
    ) -> Result<Vec<Organization>, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.list_organizations_for_user(sub).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn list_organizations(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Organization>, u64), CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.list_organizations(limit, offset).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn update_organization(
        &self,
        command: UpdateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.update_organization(command).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn update_legal_identity(
        &self,
        command: UpdateLegalIdentityCommand,
    ) -> Result<Organization, CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.update_legal_identity(command).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn soft_delete_organization(&self, id: OrganizationId) -> Result<(), CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.soft_delete_organization(id).await
    }

    #[transactional(organization, role, member, user, authz)]
    pub async fn leave_organization(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );
        service.leave_organization(organization_id, user_id).await
    }
}

/// The organization's timezone, refused rather than defaulted when unusable.
///
/// Shared by every read that turns a calendar day into instants: field
/// clocking, profitability, worked hours. Falling back to UTC would look like
/// it worked and quietly file evening work under the wrong day, which is the
/// bug the column exists to prevent.
///
/// Takes the repository rather than `&self` so a caller already inside a
/// `#[transactional]` block reuses that transaction instead of opening a
/// second one.
pub(crate) async fn resolve_timezone(
    organizations: &mut impl crate::domain::organization::ports::OrganizationRepository,
    organization_id: OrganizationId,
) -> Result<crate::Tz, CoreError> {
    let name = organizations
        .find_timezone(organization_id)
        .await?
        .ok_or(CoreError::NotFound)?;

    name.parse::<crate::Tz>().map_err(|_| {
        CoreError::Internal(format!(
            "organization timezone `{name}` is not an IANA zone"
        ))
    })
}
