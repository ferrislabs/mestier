use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::domain::{
    organization::OrganizationId,
    role::{Role, RoleId, commands::CreateRoleCommand, ports::RoleRepository},
};

pub struct RoleService<R>
where
    R: RoleRepository,
{
    repo: R,
}

impl<R> RoleService<R>
where
    R: RoleRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self), fields(organization_id = %command.organization_id.0, role.name = %command.name), err)]
    pub async fn create_role(&mut self, command: CreateRoleCommand) -> Result<Role, CoreError> {
        let now = Utc::now();
        let role = Role {
            id: RoleId(generate_uuid_v7()),
            organization_id: command.organization_id,
            name: command.name,
            permissions: command.permissions,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&role).await
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0), err)]
    pub async fn list_roles(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Role>, CoreError> {
        self.repo.list_by_organization(organization_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::role::{Permissions, ports::MockRoleRepository};
    use chrono::Utc;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org_id() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    #[tokio::test]
    async fn create_role_persists_via_repo() {
        let mut repo = MockRoleRepository::new();
        repo.expect_insert().times(1).returning(|r| {
            let cloned = Role {
                id: r.id,
                organization_id: r.organization_id,
                name: r.name.clone(),
                permissions: r.permissions,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = RoleService::new(repo);
        let role = service
            .create_role(CreateRoleCommand {
                organization_id: org_id(),
                name: "admin".into(),
                permissions: Permissions::MANAGE_MEMBERS,
            })
            .await
            .unwrap();

        assert_eq!(role.name, "admin");
        assert_eq!(role.permissions, Permissions::MANAGE_MEMBERS);
    }

    #[tokio::test]
    async fn list_roles_delegates_to_repo() {
        let id = org_id();
        let mut repo = MockRoleRepository::new();
        repo.expect_list_by_organization()
            .with(eq(id))
            .times(1)
            .returning(move |oid| {
                let now = Utc::now();
                let roles = vec![Role {
                    id: RoleId(Uuid::new_v4()),
                    organization_id: oid,
                    name: "admin".into(),
                    permissions: Permissions::MANAGE_MEMBERS,
                    created_at: now,
                    updated_at: now,
                }];
                Box::pin(async move { Ok(roles) })
            });

        let mut service = RoleService::new(repo);
        let roles = service.list_roles(id).await.unwrap();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "admin");
    }
}
