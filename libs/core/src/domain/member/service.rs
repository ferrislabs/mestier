use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    UserId,
    domain::{
        member::{
            Member, MemberId,
            commands::{
                AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UpdateMemberCommand,
            },
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
        validate_last_name(&command.last_name)?;
        validate_first_name(&command.first_name)?;

        let now = Utc::now();
        let member = Member {
            id: MemberId(generate_uuid_v7()),
            organization_id: command.organization_id,
            user_id: Some(command.user_id),
            last_name: command.last_name,
            first_name: command.first_name,
            joined_at: Some(now),
            created_at: now,
            deleted_at: None,
        };

        self.repo.insert(&member).await
    }

    /// A free, named seat: `user_id` is `None` and stays that way until an
    /// invitation (#184) fills it, so `joined_at` is `None` too.
    #[tracing::instrument(skip(self), fields(organization_id = %command.organization_id.0), err)]
    pub async fn create_member(
        &mut self,
        command: CreateMemberCommand,
    ) -> Result<Member, CoreError> {
        validate_last_name(&command.last_name)?;
        validate_first_name(&command.first_name)?;

        let now = Utc::now();
        let member = Member {
            id: MemberId(generate_uuid_v7()),
            organization_id: command.organization_id,
            user_id: None,
            last_name: command.last_name,
            first_name: command.first_name,
            joined_at: None,
            created_at: now,
            deleted_at: None,
        };

        self.repo.insert(&member).await
    }

    #[tracing::instrument(skip(self), fields(member_id = %command.member_id.0), err)]
    pub async fn update_member(
        &mut self,
        command: UpdateMemberCommand,
    ) -> Result<Member, CoreError> {
        validate_last_name(&command.last_name)?;
        validate_first_name(&command.first_name)?;

        let mut member = self
            .repo
            .find_by_id(command.member_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        member.last_name = command.last_name;
        member.first_name = command.first_name;

        self.repo.update(&member).await
    }

    /// Mirrors `EmployeeService::get_employee`: a member fetched by id must
    /// exist, or the caller gets `NotFound` rather than an `Option` to
    /// unwrap at every call site.
    #[tracing::instrument(skip(self), fields(member_id = %member_id.0), err)]
    pub async fn get_member(&mut self, member_id: MemberId) -> Result<Member, CoreError> {
        self.repo
            .find_by_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0), err)]
    pub async fn list_members(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Member>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
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
        self.repo.soft_delete(member_id, Utc::now()).await
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

/// Mirrors `employee::service::validate_last_name`: a nameless seat would be
/// a ghost in the planning grid.
fn validate_last_name(last_name: &str) -> Result<(), CoreError> {
    if last_name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "member last name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

/// `first_name` is optional, but when it is provided it must not be
/// blank — mirroring `chk_members_first_name_not_blank_when_present`.
fn validate_first_name(first_name: &Option<String>) -> Result<(), CoreError> {
    if first_name.as_deref().is_some_and(|v| v.trim().is_empty()) {
        return Err(CoreError::Conflict(
            "member first name cannot be blank when provided".to_owned(),
        ));
    }

    Ok(())
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

    fn add_command() -> AddMemberCommand {
        AddMemberCommand {
            organization_id: org_id(),
            user_id: user_id(),
            last_name: "Alice".to_owned(),
            first_name: None,
        }
    }

    fn create_command() -> CreateMemberCommand {
        CreateMemberCommand {
            organization_id: org_id(),
            last_name: "Alice".to_owned(),
            first_name: None,
        }
    }

    fn member(id: MemberId) -> Member {
        let now = Utc::now();
        Member {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            user_id: None,
            last_name: "Alice".to_owned(),
            first_name: None,
            joined_at: None,
            created_at: now,
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn add_member_persists_a_named_occupied_seat_via_repo() {
        let mut repo = MockMemberRepository::new();
        repo.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = MemberService::new(repo);
        let command = add_command();
        let oid = command.organization_id;
        let uid = command.user_id;

        let member = service.add_member(command).await.unwrap();

        assert_eq!(member.organization_id, oid);
        assert_eq!(member.user_id, Some(uid));
        assert_eq!(member.last_name, "Alice");
        assert!(member.joined_at.is_some());
    }

    #[tokio::test]
    async fn add_member_rejects_an_empty_last_name() {
        let repo = MockMemberRepository::new();
        let mut service = MemberService::new(repo);

        let err = service
            .add_member(AddMemberCommand {
                last_name: "   ".to_owned(),
                ..add_command()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn add_member_rejects_a_blank_first_name_when_provided() {
        let repo = MockMemberRepository::new();
        let mut service = MemberService::new(repo);

        let err = service
            .add_member(AddMemberCommand {
                first_name: Some("   ".to_owned()),
                ..add_command()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_member_persists_a_free_named_seat() {
        let mut repo = MockMemberRepository::new();
        repo.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = MemberService::new(repo);
        let member = service.create_member(create_command()).await.unwrap();

        assert_eq!(member.user_id, None);
        assert_eq!(member.joined_at, None);
        assert_eq!(member.last_name, "Alice");
    }

    #[tokio::test]
    async fn create_member_rejects_an_empty_last_name() {
        let repo = MockMemberRepository::new();
        let mut service = MemberService::new(repo);

        let err = service
            .create_member(CreateMemberCommand {
                last_name: "   ".to_owned(),
                ..create_command()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_member_rejects_a_blank_first_name_when_provided() {
        let repo = MockMemberRepository::new();
        let mut service = MemberService::new(repo);

        let err = service
            .create_member(CreateMemberCommand {
                first_name: Some("   ".to_owned()),
                ..create_command()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn update_member_mutates_the_name_of_an_existing_member() {
        let id = MemberId(Uuid::new_v4());
        let mut repo = MockMemberRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id))) }));
        repo.expect_update().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = MemberService::new(repo);
        let updated = service
            .update_member(UpdateMemberCommand {
                member_id: id,
                last_name: "Bob".to_owned(),
                first_name: Some("Martin".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(updated.last_name, "Bob");
        assert_eq!(updated.first_name.as_deref(), Some("Martin"));
    }

    #[tokio::test]
    async fn update_member_returns_not_found_when_missing() {
        let id = MemberId(Uuid::new_v4());
        let mut repo = MockMemberRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = MemberService::new(repo);
        let err = service
            .update_member(UpdateMemberCommand {
                member_id: id,
                last_name: "Bob".to_owned(),
                first_name: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn update_member_rejects_an_empty_last_name() {
        let id = MemberId(Uuid::new_v4());
        let repo = MockMemberRepository::new();
        let mut service = MemberService::new(repo);

        let err = service
            .update_member(UpdateMemberCommand {
                member_id: id,
                last_name: "   ".to_owned(),
                first_name: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn list_members_delegates_to_repo_with_pagination() {
        let oid = org_id();
        let mut repo = MockMemberRepository::new();
        repo.expect_list_by_organization()
            .with(eq(oid), eq(20u64), eq(40u64))
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok((vec![], 0)) }));

        let mut service = MemberService::new(repo);
        let (members, total) = service.list_members(oid, 20, 40).await.unwrap();

        assert!(members.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn get_member_returns_the_member_when_present() {
        let id = MemberId(Uuid::new_v4());
        let mut repo = MockMemberRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id))) }));

        let mut service = MemberService::new(repo);
        let found = service.get_member(id).await.unwrap();

        assert_eq!(found.id, id);
    }

    #[tokio::test]
    async fn get_member_returns_not_found_when_absent() {
        let id = MemberId(Uuid::new_v4());
        let mut repo = MockMemberRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = MemberService::new(repo);
        let err = service.get_member(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
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
    async fn remove_member_soft_deletes_via_repo() {
        let mid = MemberId(Uuid::new_v4());
        let mut repo = MockMemberRepository::new();
        repo.expect_soft_delete()
            .withf(move |id, _| *id == mid)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

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
