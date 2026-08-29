use std::collections::HashMap;

use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    UserId,
    application::policy,
    domain::{
        member::{
            Member, MemberId, MemberWithAccount,
            commands::{
                AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UpdateMemberCommand,
            },
            ports::MemberRepository,
        },
        organization::OrganizationId,
        role::{RoleId, ports::RoleRepository},
        user::{User, ports::UserRepository},
    },
};

pub struct MemberService<M, R, U, A>
where
    M: MemberRepository,
    R: RoleRepository,
    U: UserRepository,
    A: Authorizer,
{
    member_repository: M,
    role_repository: R,
    user_repository: U,
    authz: A,
}

impl<M, R, U, A> MemberService<M, R, U, A>
where
    M: MemberRepository,
    R: RoleRepository,
    U: UserRepository,
    A: Authorizer,
{
    pub fn new(member_repository: M, role_repository: R, user_repository: U, authz: A) -> Self {
        Self {
            member_repository,
            role_repository,
            user_repository,
            authz,
        }
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

        self.member_repository.insert(&member).await
    }

    /// A free, named seat: `user_id` is `None` and stays that way until an
    /// invitation (#184) fills it, so `joined_at` is `None` too.
    ///
    /// No resource to load — the target organization comes straight from
    /// the request path (`POST /organizations/{organization_id}/members`
    /// is a nested route, not a bare id, so the path *is* the resource
    /// context; see `update_member`/`remove_member` for the bare-id case).
    #[tracing::instrument(skip(self), fields(organization_id = %command.organization_id.0), err)]
    pub async fn create_member(
        &mut self,
        command: CreateMemberCommand,
    ) -> Result<Member, CoreError> {
        // 1/2. Authorize — enrich the actor with the org membership /
        //      aggregated permission bitfield, then ask the policy engine.
        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "member.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        // 3. Validate + mutate.
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

        self.member_repository.insert(&member).await
    }

    #[tracing::instrument(skip(self), fields(member_id = %command.member_id.0), err)]
    pub async fn update_member(
        &mut self,
        command: UpdateMemberCommand,
    ) -> Result<MemberWithAccount, CoreError> {
        // 1. Load — required to authorize against actual org context, and
        //    to fall the PATCH's absent fields back to what is stored.
        let mut member = self
            .member_repository
            .find_by_id(command.member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // 2. Authorize.
        let actor = policy::enrich_for_organization(
            command.actor,
            member.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "member.manage",
            Resource::new("member", member.id.0.to_string()),
        )
        .await?;

        // 3. Merge the partial update onto the loaded seat — this is the
        //    read side of PATCH semantics, which used to live in the
        //    handler before it had a member to load. `None` leaves the
        //    stored value untouched; `first_name`'s outer `None` does the
        //    same, `Some(None)` clears it.
        let last_name = command
            .last_name
            .unwrap_or_else(|| member.last_name.clone());
        let first_name = match command.first_name {
            Some(value) => value,
            None => member.first_name.clone(),
        };
        validate_last_name(&last_name)?;
        validate_first_name(&first_name)?;

        member.last_name = last_name;
        member.first_name = first_name;

        let updated = self.member_repository.update(&member).await?;
        let account = self.hydrate_account(updated.user_id).await?;

        Ok(MemberWithAccount {
            member: updated,
            account,
        })
    }

    /// Mirrors `EmployeeService::get_employee`: a member fetched by id must
    /// exist, or the caller gets `NotFound` rather than an `Option` to
    /// unwrap at every call site.
    ///
    /// Loads first, then authorizes against the *loaded* seat's
    /// organization — never a caller-supplied one — so a bare
    /// `/members/{id}` route can never be turned into a cross-tenant IDOR.
    #[tracing::instrument(skip(self), fields(member_id = %member_id.0), err)]
    pub async fn get_member(
        &mut self,
        actor: Subject,
        member_id: MemberId,
    ) -> Result<MemberWithAccount, CoreError> {
        // 1. Load.
        let member = self
            .member_repository
            .find_by_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // 2. Authorize — reads need membership only. There is no "read a
        //    member" action in the action -> permission-bits table, and
        //    inventing one would be a bit to manage for nothing;
        //    `enrich_for_organization` alone already returns
        //    `CoreError::Forbidden` for a non-member.
        policy::enrich_for_organization(
            actor,
            member.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;

        // 3. Hydrate the occupant.
        let account = self.hydrate_account(member.user_id).await?;

        Ok(MemberWithAccount { member, account })
    }

    /// No per-row load: the organization comes straight from the request
    /// path, exactly like `create_member` (this is the collection route,
    /// not a bare id).
    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0), err)]
    pub async fn list_members(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<MemberWithAccount>, u64), CoreError> {
        // 1/2. Authorize — reads need membership only, see `get_member`.
        policy::enrich_for_organization(
            actor,
            organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;

        // 3. Fetch the page, then hydrate every occupied seat's account in
        //    a single query — never one lookup per row.
        let (members, total) = self
            .member_repository
            .list_by_organization(organization_id, limit, offset)
            .await?;

        let user_ids: Vec<UserId> = members.iter().filter_map(|member| member.user_id).collect();
        let accounts_by_id: HashMap<UserId, User> = if user_ids.is_empty() {
            HashMap::new()
        } else {
            self.user_repository
                .list_by_ids(&user_ids)
                .await?
                .into_iter()
                .map(|user| (user.id, user))
                .collect()
        };

        let items = members
            .into_iter()
            .map(|member| {
                let account = member
                    .user_id
                    .and_then(|id| accounts_by_id.get(&id).cloned());
                MemberWithAccount { member, account }
            })
            .collect();

        Ok((items, total))
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0, user_id = %user_id.0), err)]
    pub async fn find_membership(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Member>, CoreError> {
        self.member_repository
            .find_by_org_and_user(organization_id, user_id)
            .await
    }

    /// Loads first, then authorizes against the *loaded* seat's
    /// organization — never a caller-supplied one, mirroring `get_member`.
    /// Returns the removed seat as it stood the instant before deletion —
    /// its `deleted_at` is not backfilled onto the returned value, since the
    /// caller (`MestierUseCase::remove_member`) only needs it to build the
    /// `member.removed` event, not to display it.
    #[tracing::instrument(skip(self), fields(member_id = %member_id.0), err)]
    pub async fn remove_member(
        &mut self,
        actor: Subject,
        member_id: MemberId,
    ) -> Result<Member, CoreError> {
        // 1. Load.
        let member = self
            .member_repository
            .find_by_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // 2. Authorize.
        let actor = policy::enrich_for_organization(
            actor,
            member.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "member.remove",
            Resource::new("member", member.id.0.to_string()),
        )
        .await?;

        // 3. Mutate.
        self.member_repository
            .soft_delete(member_id, Utc::now())
            .await?;

        Ok(member)
    }

    #[tracing::instrument(skip(self), fields(member_id = %command.member_id.0, role_id = %command.role_id.0), err)]
    pub async fn assign_role(&mut self, command: AssignRoleCommand) -> Result<(), CoreError> {
        let member = self
            .member_repository
            .find_by_id(command.member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            command.actor,
            member.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "role.assign",
            Resource::new("organization", member.organization_id.0.to_string()),
        )
        .await?;

        self.member_repository
            .assign_role(command.member_id, command.role_id)
            .await
    }

    #[tracing::instrument(skip(self, actor), fields(member_id = %member_id.0), err)]
    pub async fn list_role_ids(
        &mut self,
        member_id: MemberId,
        actor: Subject,
    ) -> Result<Vec<RoleId>, CoreError> {
        let member = self
            .member_repository
            .find_by_id(member_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            member.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "role.manage",
            Resource::new("organization", member.organization_id.0.to_string()),
        )
        .await?;

        self.member_repository.list_role_ids(member_id).await
    }

    /// Resolves the account behind a single occupied seat. `None` when the
    /// seat is vacant (`user_id: None`) — no query needed — or when the
    /// linked user row cannot be found (never surfaced as an error: a
    /// member response degrades to a vacant-looking seat rather than
    /// failing the whole request).
    async fn hydrate_account(
        &mut self,
        user_id: Option<UserId>,
    ) -> Result<Option<User>, CoreError> {
        let Some(user_id) = user_id else {
            return Ok(None);
        };

        self.user_repository.find_by_id(user_id).await
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
    use crate::{
        UserId,
        domain::{
            member::ports::MockMemberRepository, role::ports::MockRoleRepository,
            user::ports::MockUserRepository,
        },
    };
    use authz::{Decision, MockAuthorizer};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org_id() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn user_id() -> UserId {
        UserId(Uuid::new_v4())
    }

    /// Subject built the way the API handler will build it: `mestier.user_id`
    /// is set, no IAM roles, no org context yet (the service enriches it).
    fn actor_for(user_id: UserId) -> Subject {
        policy::user_subject(user_id, Vec::new())
    }

    fn add_command() -> AddMemberCommand {
        AddMemberCommand {
            organization_id: org_id(),
            user_id: user_id(),
            last_name: "Alice".to_owned(),
            first_name: None,
        }
    }

    fn create_command(actor: Subject, organization_id: OrganizationId) -> CreateMemberCommand {
        CreateMemberCommand {
            actor,
            organization_id,
            last_name: "Alice".to_owned(),
            first_name: None,
        }
    }

    fn member(id: MemberId, organization_id: OrganizationId) -> Member {
        let now = Utc::now();
        Member {
            id,
            organization_id,
            user_id: None,
            last_name: "Alice".to_owned(),
            first_name: None,
            joined_at: None,
            created_at: now,
            deleted_at: None,
        }
    }

    fn user(id: UserId) -> User {
        let now = Utc::now();
        User {
            id,
            email: "alice@example.com".to_owned(),
            username: "alice".to_owned(),
            name: "Alice".to_owned(),
            sub: "sub-alice".to_owned(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Stages the calls `policy::enrich_for_organization` makes: lookup
    /// the member, list its role ids, list the org's roles. The roles
    /// list is left empty so aggregated permissions = 0; tests that rely
    /// on the engine answering `allow`/`deny` go through `MockAuthorizer`.
    fn stage_org_membership(
        members: &mut MockMemberRepository,
        roles: &mut MockRoleRepository,
        org_id: OrganizationId,
        user_id: UserId,
        acting_member_id: MemberId,
    ) {
        members
            .expect_find_by_org_and_user()
            .with(eq(org_id), eq(user_id))
            .times(1)
            .returning(move |organization_id, user_id| {
                let m = Member {
                    id: acting_member_id,
                    organization_id,
                    user_id: Some(user_id),
                    last_name: "Actor".to_owned(),
                    first_name: None,
                    joined_at: Some(Utc::now()),
                    created_at: Utc::now(),
                    deleted_at: None,
                };
                Box::pin(async move { Ok(Some(m)) })
            });
        members
            .expect_list_role_ids()
            .with(eq(acting_member_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        roles
            .expect_list_by_organization()
            .with(eq(org_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
    }

    fn stage_not_a_member(
        members: &mut MockMemberRepository,
        org_id: OrganizationId,
        user_id: UserId,
    ) {
        members
            .expect_find_by_org_and_user()
            .with(eq(org_id), eq(user_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
    }

    #[tokio::test]
    async fn add_member_persists_a_named_occupied_seat_via_repo() {
        let mut member_repository = MockMemberRepository::new();
        member_repository.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
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
        let mut service = MemberService::new(
            MockMemberRepository::new(),
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

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
        let mut service = MemberService::new(
            MockMemberRepository::new(),
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

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
    async fn create_member_persists_a_free_named_seat_when_authorized() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );

        let member = service
            .create_member(create_command(actor_for(uid), oid))
            .await
            .unwrap();

        assert_eq!(member.user_id, None);
        assert_eq!(member.joined_at, None);
        assert_eq!(member.last_name, "Alice");
        assert_eq!(member.organization_id, oid);
    }

    #[tokio::test]
    async fn create_member_returns_forbidden_when_not_a_member() {
        let oid = org_id();
        let uid = user_id();

        let mut member_repository = MockMemberRepository::new();
        stage_not_a_member(&mut member_repository, oid, uid);

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let err = service
            .create_member(create_command(actor_for(uid), oid))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn create_member_returns_forbidden_when_authz_denies() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        // No `expect_insert` — the call must short-circuit before mutation.

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .withf(move |req| {
                req.action.name == "member.manage"
                    && req.resource.r#type == "organization"
                    && req.resource.id == oid.0.to_string()
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::deny()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );

        let err = service
            .create_member(create_command(actor_for(uid), oid))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn create_member_rejects_an_empty_last_name() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        // No `expect_insert` — validation fails before the repo is touched.

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );

        let err = service
            .create_member(CreateMemberCommand {
                last_name: "   ".to_owned(),
                ..create_command(actor_for(uid), oid)
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_member_rejects_a_blank_first_name_when_provided() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );

        let err = service
            .create_member(CreateMemberCommand {
                first_name: Some("   ".to_owned()),
                ..create_command(actor_for(uid), oid)
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn update_member_mutates_the_name_when_authorized() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository.expect_update().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );
        let updated = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(uid),
                member_id: id,
                last_name: Some("Bob".to_owned()),
                first_name: Some(Some("Martin".to_owned())),
            })
            .await
            .unwrap();

        assert_eq!(updated.member.last_name, "Bob");
        assert_eq!(updated.member.first_name.as_deref(), Some("Martin"));
        assert!(updated.account.is_none());
    }

    #[tokio::test]
    async fn update_member_returns_hydrated_account_when_seat_is_occupied() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let occupant_id = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| {
                let mut m = member(id, oid);
                m.user_id = Some(occupant_id);
                Box::pin(async move { Ok(Some(m)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository.expect_update().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(eq(occupant_id))
            .times(1)
            .returning(move |id| {
                let u = user(id);
                Box::pin(async move { Ok(Some(u)) })
            });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service =
            MemberService::new(member_repository, role_repository, user_repository, authz);

        let updated = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(uid),
                member_id: id,
                last_name: Some("Bob".to_owned()),
                first_name: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.account.map(|a| a.id), Some(occupant_id));
    }

    #[tokio::test]
    async fn update_member_leaves_last_name_untouched_when_absent() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository.expect_update().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );
        let updated = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(uid),
                member_id: id,
                last_name: None,
                first_name: None,
            })
            .await
            .unwrap();

        // `member(id, oid)`'s fixture last name is "Alice" — must survive.
        assert_eq!(updated.member.last_name, "Alice");
    }

    #[tokio::test]
    async fn update_member_clears_first_name_when_explicitly_null() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| {
                let mut m = member(id, oid);
                m.first_name = Some("Baptiste".to_owned());
                Box::pin(async move { Ok(Some(m)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository.expect_update().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );
        let updated = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(uid),
                member_id: id,
                last_name: None,
                first_name: Some(None),
            })
            .await
            .unwrap();

        assert_eq!(updated.member.first_name, None);
    }

    #[tokio::test]
    async fn update_member_returns_not_found_when_missing() {
        let id = MemberId(Uuid::new_v4());
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let err = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(user_id()),
                member_id: id,
                last_name: Some("Bob".to_owned()),
                first_name: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn update_member_returns_forbidden_when_not_a_member() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_not_a_member(&mut member_repository, oid, uid);

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let err = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(uid),
                member_id: id,
                last_name: Some("Bob".to_owned()),
                first_name: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn update_member_returns_forbidden_when_authz_denies() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        // No `expect_update` — the call must short-circuit before mutation.

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .withf(move |req| {
                req.action.name == "member.manage"
                    && req.resource.r#type == "member"
                    && req.resource.id == id.0.to_string()
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::deny()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );
        let err = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(uid),
                member_id: id,
                last_name: Some("Bob".to_owned()),
                first_name: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn update_member_rejects_an_empty_last_name() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        // No `expect_update` — validation fails before the repo is touched.

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );
        let err = service
            .update_member(UpdateMemberCommand {
                actor: actor_for(uid),
                member_id: id,
                last_name: Some("   ".to_owned()),
                first_name: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn list_members_delegates_to_repo_with_pagination_when_authorized() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository
            .expect_list_by_organization()
            .with(eq(oid), eq(20u64), eq(40u64))
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok((vec![], 0)) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let (members, total) = service
            .list_members(actor_for(uid), oid, 20, 40)
            .await
            .unwrap();

        assert!(members.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn list_members_hydrates_accounts_in_a_single_query() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());
        let occupant_a = user_id();
        let occupant_b = user_id();

        let vacant = member(MemberId(Uuid::new_v4()), oid);
        let mut occupied_a = member(MemberId(Uuid::new_v4()), oid);
        occupied_a.user_id = Some(occupant_a);
        let mut occupied_b = member(MemberId(Uuid::new_v4()), oid);
        occupied_b.user_id = Some(occupant_b);
        let members = vec![vacant, occupied_a, occupied_b];

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository
            .expect_list_by_organization()
            .with(eq(oid), eq(10u64), eq(0u64))
            .times(1)
            .returning(move |_, _, _| {
                let cloned = members.clone();
                Box::pin(async move { Ok((cloned, 3)) })
            });

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_list_by_ids()
            .withf(move |ids| {
                let mut ids = ids.to_vec();
                ids.sort_by_key(|id| id.0);
                let mut expected = vec![occupant_a, occupant_b];
                expected.sort_by_key(|id| id.0);
                ids == expected
            })
            .times(1)
            .returning(move |ids| {
                let users: Vec<User> = ids.iter().map(|id| user(*id)).collect();
                Box::pin(async move { Ok(users) })
            });

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let (items, total) = service
            .list_members(actor_for(uid), oid, 10, 0)
            .await
            .unwrap();

        assert_eq!(total, 3);
        assert_eq!(items.len(), 3);
        let occupied_accounts: Vec<UserId> = items
            .iter()
            .filter_map(|item| item.account.as_ref().map(|a| a.id))
            .collect();
        assert_eq!(occupied_accounts.len(), 2);
        assert!(occupied_accounts.contains(&occupant_a));
        assert!(occupied_accounts.contains(&occupant_b));
    }

    #[tokio::test]
    async fn list_members_returns_forbidden_when_not_a_member() {
        let oid = org_id();
        let uid = user_id();

        let mut member_repository = MockMemberRepository::new();
        stage_not_a_member(&mut member_repository, oid, uid);
        // No `expect_list_by_organization` — the call must short-circuit.

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let err = service
            .list_members(actor_for(uid), oid, 20, 0)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn get_member_returns_the_hydrated_seat_and_account_when_authorized() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let occupant_id = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| {
                let mut m = member(id, oid);
                m.user_id = Some(occupant_id);
                Box::pin(async move { Ok(Some(m)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(eq(occupant_id))
            .times(1)
            .returning(move |id| {
                let u = user(id);
                Box::pin(async move { Ok(Some(u)) })
            });

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let found = service.get_member(actor_for(uid), id).await.unwrap();

        assert_eq!(found.member.id, id);
        assert_eq!(found.account.map(|a| a.id), Some(occupant_id));
    }

    #[tokio::test]
    async fn get_member_returns_no_account_when_seat_is_vacant() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        // No `expect_find_by_id` on `user_repository`: a vacant seat needs
        // no lookup at all.

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let found = service.get_member(actor_for(uid), id).await.unwrap();

        assert!(found.account.is_none());
    }

    #[tokio::test]
    async fn get_member_returns_not_found_when_absent() {
        let id = MemberId(Uuid::new_v4());
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let err = service
            .get_member(actor_for(user_id()), id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn get_member_returns_forbidden_when_not_a_member() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_not_a_member(&mut member_repository, oid, uid);

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let err = service.get_member(actor_for(uid), id).await.unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn find_membership_returns_optional_member() {
        let oid = org_id();
        let uid = user_id();

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(oid), eq(uid))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let result = service.find_membership(oid, uid).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn remove_member_soft_deletes_when_authorized() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        member_repository
            .expect_soft_delete()
            .withf(move |i, _| *i == id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );
        let removed = service.remove_member(actor_for(uid), id).await.unwrap();

        assert_eq!(removed.id, id);
    }

    #[tokio::test]
    async fn remove_member_returns_not_found_when_missing() {
        let id = MemberId(Uuid::new_v4());
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let err = service
            .remove_member(actor_for(user_id()), id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn remove_member_returns_forbidden_when_not_a_member() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_not_a_member(&mut member_repository, oid, uid);

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );
        let err = service.remove_member(actor_for(uid), id).await.unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn remove_member_returns_forbidden_when_authz_denies() {
        let id = MemberId(Uuid::new_v4());
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        let mut role_repository = MockRoleRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(id, oid))) }));
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            oid,
            uid,
            acting_member_id,
        );
        // No `expect_soft_delete` — the call must short-circuit.

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .withf(move |req| {
                req.action.name == "member.remove"
                    && req.resource.r#type == "member"
                    && req.resource.id == id.0.to_string()
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::deny()) }));

        let mut service = MemberService::new(
            member_repository,
            role_repository,
            MockUserRepository::new(),
            authz,
        );
        let err = service.remove_member(actor_for(uid), id).await.unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    fn allow_once(authz: &mut MockAuthorizer) {
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    }

    #[tokio::test]
    async fn assign_role_calls_repo() {
        let organization_id = org_id();
        let mid = MemberId(Uuid::new_v4());
        let rid = RoleId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(mid))
            .times(1)
            .returning(move |_| {
                let m = member(mid, organization_id);
                Box::pin(async move { Ok(Some(m)) })
            });
        member_repository
            .expect_assign_role()
            .with(eq(mid), eq(rid))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            authz,
        );
        service
            .assign_role(AssignRoleCommand {
                actor: Subject::system(),
                member_id: mid,
                role_id: rid,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn list_role_ids_delegates_to_repo() {
        let organization_id = org_id();
        let mid = MemberId(Uuid::new_v4());
        let returned = RoleId(Uuid::new_v4());

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .with(eq(mid))
            .times(1)
            .returning(move |_| {
                let m = member(mid, organization_id);
                Box::pin(async move { Ok(Some(m)) })
            });
        member_repository
            .expect_list_role_ids()
            .with(eq(mid))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(vec![returned]) }));
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = MemberService::new(
            member_repository,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            authz,
        );
        let ids = service.list_role_ids(mid, Subject::system()).await.unwrap();

        assert_eq!(ids, vec![returned]);
    }
}
