use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    application::policy,
    domain::{
        member::ports::MemberRepository,
        organization::OrganizationId,
        role::{
            Role, RoleId,
            commands::{CreateRoleCommand, UpdateRoleCommand},
            ports::RoleRepository,
        },
    },
};

pub struct RoleService<R, M, A>
where
    R: RoleRepository,
    M: MemberRepository,
    A: Authorizer,
{
    repo: R,
    member_repository: M,
    authz: A,
}

impl<R, M, A> RoleService<R, M, A>
where
    R: RoleRepository,
    M: MemberRepository,
    A: Authorizer,
{
    pub fn new(repo: R, member_repository: M, authz: A) -> Self {
        Self {
            repo,
            member_repository,
            authz,
        }
    }

    #[tracing::instrument(skip(self), fields(organization_id = %command.organization_id.0, role.name = %command.name), err)]
    pub async fn create_role(&mut self, command: CreateRoleCommand) -> Result<Role, CoreError> {
        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut self.member_repository,
            &mut self.repo,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "role.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let now = Utc::now();
        let role = Role {
            id: RoleId(generate_uuid_v7()),
            organization_id: command.organization_id,
            name: command.name,
            permissions: command.permissions,
            // A caller can only ever reach this constructor, never the
            // seeding path in `organization::service::create_organization` —
            // a role it creates is never one of the three protected ones.
            is_seeded: false,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&role).await
    }

    #[tracing::instrument(skip(self, actor), fields(organization_id = %organization_id.0), err)]
    pub async fn list_roles(
        &mut self,
        organization_id: OrganizationId,
        actor: Subject,
    ) -> Result<Vec<Role>, CoreError> {
        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut self.member_repository,
            &mut self.repo,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "role.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await?;

        self.repo.list_by_organization(organization_id).await
    }

    /// Refuses to rename a seeded role (its permissions stay editable) —
    /// see [`Role::is_seeded`]'s own doc for why the name has to be fixed.
    #[tracing::instrument(skip(self), fields(role_id = %command.role_id.0), err)]
    pub async fn update_role(&mut self, command: UpdateRoleCommand) -> Result<Role, CoreError> {
        let mut role = self
            .repo
            .find_by_id(command.role_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            command.actor,
            role.organization_id,
            &mut self.member_repository,
            &mut self.repo,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "role.manage",
            Resource::new("organization", role.organization_id.0.to_string()),
        )
        .await?;

        if role.is_seeded && role.name != command.name {
            return Err(CoreError::Conflict(format!(
                "role '{}' is seeded and its name cannot be changed",
                role.name
            )));
        }

        role.name = command.name;
        role.permissions = command.permissions;
        role.updated_at = Utc::now();

        self.repo.update(&role).await
    }

    /// Refuses a seeded role outright, and any other role still assigned to
    /// a member — `member_roles.role_id` cascades on delete, so deleting an
    /// assigned role would silently strip whoever holds it rather than the
    /// organization choosing to (#308).
    #[tracing::instrument(skip(self, actor), fields(role_id = %role_id.0), err)]
    pub async fn delete_role(&mut self, role_id: RoleId, actor: Subject) -> Result<(), CoreError> {
        let role = self
            .repo
            .find_by_id(role_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            role.organization_id,
            &mut self.member_repository,
            &mut self.repo,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "role.manage",
            Resource::new("organization", role.organization_id.0.to_string()),
        )
        .await?;

        if role.is_seeded {
            return Err(CoreError::Conflict(format!(
                "role '{}' is seeded and cannot be deleted",
                role.name
            )));
        }

        let assigned = self.repo.count_assigned_members(role_id).await?;
        if assigned > 0 {
            return Err(CoreError::Conflict(format!(
                "role '{}' is still assigned to {assigned} member(s); reassign them before deleting it",
                role.name
            )));
        }

        self.repo.delete(role_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        member::ports::MockMemberRepository, role::Permissions, role::ports::MockRoleRepository,
    };
    use authz::{Decision, MockAuthorizer};
    use chrono::Utc;
    use uuid::Uuid;

    fn org_id() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn system_actor() -> Subject {
        Subject::system()
    }

    fn allow_once(authz: &mut MockAuthorizer) {
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    }

    fn role(id: RoleId, organization_id: OrganizationId, is_seeded: bool) -> Role {
        let now = Utc::now();
        Role {
            id,
            organization_id,
            name: "custom".into(),
            permissions: Permissions::MANAGE_MEMBERS,
            is_seeded,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_role_persists_via_repo() {
        let organization_id = org_id();
        let mut repo = MockRoleRepository::new();
        repo.expect_insert().times(1).returning(|r| {
            let cloned = Role {
                id: r.id,
                organization_id: r.organization_id,
                name: r.name.clone(),
                permissions: r.permissions,
                is_seeded: r.is_seeded,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            Box::pin(async move { Ok(cloned) })
        });
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        let role = service
            .create_role(CreateRoleCommand {
                actor: system_actor(),
                organization_id,
                name: "admin".into(),
                permissions: Permissions::MANAGE_MEMBERS,
            })
            .await
            .unwrap();

        assert_eq!(role.name, "admin");
        assert_eq!(role.permissions, Permissions::MANAGE_MEMBERS);
        assert!(!role.is_seeded);
    }

    #[tokio::test]
    async fn list_roles_delegates_to_repo() {
        let id = org_id();
        let mut repo = MockRoleRepository::new();
        repo.expect_list_by_organization()
            .times(1)
            .returning(move |oid| {
                let roles = vec![role(RoleId(Uuid::new_v4()), oid, false)];
                Box::pin(async move { Ok(roles) })
            });
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        let roles = service.list_roles(id, system_actor()).await.unwrap();

        assert_eq!(roles.len(), 1);
    }

    #[tokio::test]
    async fn update_role_persists_new_name_and_permissions() {
        let organization_id = org_id();
        let role_id = RoleId(Uuid::new_v4());
        let existing = role(role_id, organization_id, false);
        let mut repo = MockRoleRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let existing = existing.clone();
            Box::pin(async move { Ok(Some(existing)) })
        });
        repo.expect_update().times(1).returning(|r| {
            let cloned = r.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        let updated = service
            .update_role(UpdateRoleCommand {
                actor: system_actor(),
                role_id,
                name: "foreman".into(),
                permissions: Permissions::VIEW_PLANNING,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "foreman");
        assert_eq!(updated.permissions, Permissions::VIEW_PLANNING);
    }

    #[tokio::test]
    async fn update_role_refuses_to_rename_a_seeded_role() {
        let organization_id = org_id();
        let role_id = RoleId(Uuid::new_v4());
        let mut seeded = role(role_id, organization_id, true);
        seeded.name = "owner".into();
        let mut repo = MockRoleRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let seeded = seeded.clone();
            Box::pin(async move { Ok(Some(seeded)) })
        });
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        let outcome = service
            .update_role(UpdateRoleCommand {
                actor: system_actor(),
                role_id,
                name: "not-owner".into(),
                permissions: Permissions::ALL,
            })
            .await;

        assert!(matches!(outcome, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn update_role_allows_editing_a_seeded_roles_permissions() {
        let organization_id = org_id();
        let role_id = RoleId(Uuid::new_v4());
        let mut seeded = role(role_id, organization_id, true);
        seeded.name = "admin".into();
        let mut repo = MockRoleRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let seeded = seeded.clone();
            Box::pin(async move { Ok(Some(seeded)) })
        });
        repo.expect_update().times(1).returning(|r| {
            let cloned = r.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        let updated = service
            .update_role(UpdateRoleCommand {
                actor: system_actor(),
                role_id,
                name: "admin".into(),
                permissions: Permissions::VIEW_REPORTS,
            })
            .await
            .unwrap();

        assert_eq!(updated.permissions, Permissions::VIEW_REPORTS);
    }

    #[tokio::test]
    async fn delete_role_refuses_a_seeded_role() {
        let organization_id = org_id();
        let role_id = RoleId(Uuid::new_v4());
        let seeded = role(role_id, organization_id, true);
        let mut repo = MockRoleRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let seeded = seeded.clone();
            Box::pin(async move { Ok(Some(seeded)) })
        });
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        let outcome = service.delete_role(role_id, system_actor()).await;

        assert!(matches!(outcome, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_role_refuses_a_role_still_assigned_to_members() {
        let organization_id = org_id();
        let role_id = RoleId(Uuid::new_v4());
        let custom = role(role_id, organization_id, false);
        let mut repo = MockRoleRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let custom = custom.clone();
            Box::pin(async move { Ok(Some(custom)) })
        });
        repo.expect_count_assigned_members()
            .times(1)
            .returning(|_| Box::pin(async { Ok(2) }));
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        let outcome = service.delete_role(role_id, system_actor()).await;

        assert!(matches!(outcome, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_role_deletes_an_unassigned_custom_role() {
        let organization_id = org_id();
        let role_id = RoleId(Uuid::new_v4());
        let custom = role(role_id, organization_id, false);
        let mut repo = MockRoleRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let custom = custom.clone();
            Box::pin(async move { Ok(Some(custom)) })
        });
        repo.expect_count_assigned_members()
            .times(1)
            .returning(|_| Box::pin(async { Ok(0) }));
        repo.expect_delete()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let member_repository = MockMemberRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = RoleService::new(repo, member_repository, authz);
        service.delete_role(role_id, system_actor()).await.unwrap();
    }
}
