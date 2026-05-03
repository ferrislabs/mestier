use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    UserId,
    domain::{
        member::{
            Member, MemberId,
            commands::{AddMemberCommand, AssignRoleCommand},
            ports::MemberRepository,
        },
        organization::OrganizationId,
        role::RoleId,
    },
};

pub struct MemberService<R>
where
    R: MemberRepository,
{
    repo: R,
}

impl<R> MemberService<R>
where
    R: MemberRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self), fields(organization_id = %command.organization_id.0, user_id = %command.user_id.0), err)]
    pub async fn add_member(&mut self, command: AddMemberCommand) -> Result<Member, CoreError> {
        let member = Member {
            id: MemberId(generate_uuid_v7()),
            organization_id: command.organization_id,
            user_id: command.user_id,
            joined_at: Utc::now(),
        };

        self.repo.insert(&member).await
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0), err)]
    pub async fn list_members(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Member>, CoreError> {
        self.repo.list_by_organization(organization_id).await
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0, user_id = %user_id.0), err)]
    pub async fn find_membership(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Member>, CoreError> {
        self.repo
            .find_by_org_and_user(organization_id, user_id)
            .await
    }

    #[tracing::instrument(skip(self), fields(member_id = %member_id.0), err)]
    pub async fn remove_member(&mut self, member_id: MemberId) -> Result<(), CoreError> {
        self.repo.remove(member_id).await
    }

    #[tracing::instrument(skip(self), fields(member_id = %command.member_id.0, role_id = %command.role_id.0), err)]
    pub async fn assign_role(&mut self, command: AssignRoleCommand) -> Result<(), CoreError> {
        self.repo
            .assign_role(command.member_id, command.role_id)
            .await
    }

    #[tracing::instrument(skip(self), fields(member_id = %member_id.0), err)]
    pub async fn list_role_ids(&mut self, member_id: MemberId) -> Result<Vec<RoleId>, CoreError> {
        self.repo.list_role_ids(member_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UserId, domain::member::ports::MockMemberRepository};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org_id() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn user_id() -> UserId {
        UserId(Uuid::new_v4())
    }

    #[tokio::test]
    async fn add_member_persists_via_repo() {
        let mut repo = MockMemberRepository::new();
        repo.expect_insert().times(1).returning(|m| {
            let cloned = Member {
                id: m.id,
                organization_id: m.organization_id,
                user_id: m.user_id,
                joined_at: m.joined_at,
            };
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = MemberService::new(repo);
        let oid = org_id();
        let uid = user_id();

        let member = service
            .add_member(AddMemberCommand {
                organization_id: oid,
                user_id: uid,
            })
            .await
            .unwrap();

        assert_eq!(member.organization_id, oid);
        assert_eq!(member.user_id, uid);
    }

    #[tokio::test]
    async fn list_members_delegates_to_repo() {
        let oid = org_id();
        let mut repo = MockMemberRepository::new();
        repo.expect_list_by_organization()
            .with(eq(oid))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let mut service = MemberService::new(repo);
        let members = service.list_members(oid).await.unwrap();

        assert!(members.is_empty());
    }

    #[tokio::test]
    async fn find_membership_returns_optional_member() {
        let oid = org_id();
        let uid = user_id();

        let mut repo = MockMemberRepository::new();
        repo.expect_find_by_org_and_user()
            .with(eq(oid), eq(uid))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let mut service = MemberService::new(repo);
        let result = service.find_membership(oid, uid).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn remove_member_calls_repo() {
        let mid = MemberId(Uuid::new_v4());
        let mut repo = MockMemberRepository::new();
        repo.expect_remove()
            .with(eq(mid))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut service = MemberService::new(repo);
        service.remove_member(mid).await.unwrap();
    }

    #[tokio::test]
    async fn assign_role_calls_repo() {
        let mid = MemberId(Uuid::new_v4());
        let rid = RoleId(Uuid::new_v4());

        let mut repo = MockMemberRepository::new();
        repo.expect_assign_role()
            .with(eq(mid), eq(rid))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = MemberService::new(repo);
        service
            .assign_role(AssignRoleCommand {
                member_id: mid,
                role_id: rid,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn list_role_ids_delegates_to_repo() {
        let mid = MemberId(Uuid::new_v4());
        let returned = RoleId(Uuid::new_v4());

        let mut repo = MockMemberRepository::new();
        repo.expect_list_role_ids()
            .with(eq(mid))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(vec![returned]) }));

        let mut service = MemberService::new(repo);
        let ids = service.list_role_ids(mid).await.unwrap();

        assert_eq!(ids, vec![returned]);
    }
}
