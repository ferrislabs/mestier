use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    application::policy,
    domain::{
        invitation::{
            Invitation, InvitationId,
            commands::{AcceptInvitationCommand, InviteMemberCommand, RevokeInvitationCommand},
            events::{MemberInvited, MemberJoined},
            ports::InvitationRepository,
            token,
        },
        member::{Member, MemberId, ports::MemberRepository},
        organization::OrganizationId,
        role::ports::RoleRepository,
        user::ports::UserRepository,
    },
};

pub struct InvitationService<I, M, R, U, A, E>
where
    I: InvitationRepository,
    M: MemberRepository,
    R: RoleRepository,
    U: UserRepository,
    A: Authorizer,
    E: EventEmitter,
{
    invitation_repository: I,
    member_repository: M,
    role_repository: R,
    user_repository: U,
    authz: A,
    emitter: E,
}

impl<I, M, R, U, A, E> InvitationService<I, M, R, U, A, E>
where
    I: InvitationRepository,
    M: MemberRepository,
    R: RoleRepository,
    U: UserRepository,
    A: Authorizer,
    E: EventEmitter,
{
    pub fn new(
        invitation_repository: I,
        member_repository: M,
        role_repository: R,
        user_repository: U,
        authz: A,
        emitter: E,
    ) -> Self {
        Self {
            invitation_repository,
            member_repository,
            role_repository,
            user_repository,
            authz,
            emitter,
        }
    }

    /// Returns the stored invitation together with its clear token — the
    /// one and only place that value is ever readable again (mirrors
    /// `MestierUseCase::create_credential`'s `(Credential, Vec<u8>)`).
    #[tracing::instrument(skip(self), fields(organization_id = %command.organization_id.0), err)]
    pub async fn invite_member(
        &mut self,
        command: InviteMemberCommand,
    ) -> Result<(Invitation, String), CoreError> {
        // 1/2. Authorize.
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
            "member.invite",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        // 3. Validate.
        let now = Utc::now();
        if command.expires_at <= now {
            return Err(CoreError::Conflict(
                "invitation expiry must be in the future".to_owned(),
            ));
        }
        if let Some(member_id) = command.member_id {
            let target = self
                .member_repository
                .find_by_id(member_id)
                .await?
                .ok_or(CoreError::NotFound)?;
            // Cross-org IDOR guard: a member id from another organization
            // reads as absent, not as forbidden — nothing here should tell
            // an attacker the id exists elsewhere.
            if target.organization_id != command.organization_id {
                return Err(CoreError::NotFound);
            }
            if target.user_id.is_some() {
                return Err(CoreError::Conflict("seat already occupied".to_owned()));
            }
        }

        // 4. Mutate.
        let created_by_user_id = policy::subject_user_id(&actor)?;
        let (clear_token, token_hash) = token::generate()?;
        let invitation = Invitation {
            id: InvitationId(generate_uuid_v7()),
            organization_id: command.organization_id,
            member_id: command.member_id,
            token_hash,
            expires_at: command.expires_at,
            consumed_at: None,
            consumed_by_user_id: None,
            created_by_user_id,
            created_at: now,
        };
        let inserted = self.invitation_repository.insert(&invitation).await?;

        self.emitter.emit(
            command.organization_id,
            &MemberInvited {
                invitation: inserted.clone(),
            },
        )?;

        Ok((inserted, clear_token))
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0), err)]
    pub async fn list_pending_invitations(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<Vec<Invitation>, CoreError> {
        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "member.invite",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await?;

        self.invitation_repository
            .list_pending_by_organization(organization_id)
            .await
    }

    #[tracing::instrument(skip(self), fields(invitation_id = %command.invitation_id.0), err)]
    pub async fn revoke_invitation(
        &mut self,
        command: RevokeInvitationCommand,
    ) -> Result<(), CoreError> {
        let invitation = self
            .invitation_repository
            .find_by_id(command.invitation_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            command.actor,
            invitation.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "member.invite",
            Resource::new("invitation", invitation.id.0.to_string()),
        )
        .await?;

        if !invitation.is_pending() {
            return Err(CoreError::Conflict(
                "invitation already accepted".to_owned(),
            ));
        }

        self.invitation_repository.revoke(invitation.id).await
    }

    /// No authorization against organization membership — see
    /// `AcceptInvitationCommand`'s doc comment. Standing in the target
    /// organization is exactly what this call grants, not a precondition of
    /// it.
    ///
    /// An expired, already-consumed, or unknown token all fail identically
    /// (`CoreError::NotFound`): the acceptance criterion this satisfies is
    /// that none of the three is distinguishable from another, and none
    /// leaks which organization a valid-looking token belonged to.
    #[tracing::instrument(skip(self), fields(user_id = %command.user_id.0), err)]
    pub async fn accept_invitation(
        &mut self,
        command: AcceptInvitationCommand,
    ) -> Result<Member, CoreError> {
        let token_hash = token::hash(&command.token);
        let invitation = self
            .invitation_repository
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(CoreError::NotFound)?;

        let now = Utc::now();
        if !invitation.is_pending() || invitation.is_expired(now) {
            return Err(CoreError::NotFound);
        }

        if self
            .member_repository
            .find_by_org_and_user(invitation.organization_id, command.user_id)
            .await?
            .is_some()
        {
            return Err(CoreError::Conflict(
                "already a member of this organization".to_owned(),
            ));
        }

        let member = match invitation.member_id {
            Some(member_id) => {
                let mut member = self
                    .member_repository
                    .find_by_id(member_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;
                if member.user_id.is_some() {
                    return Err(CoreError::Conflict("seat already occupied".to_owned()));
                }
                member.user_id = Some(command.user_id);
                member.joined_at = Some(now);
                self.member_repository.update(&member).await?
            }
            None => {
                let account = self
                    .user_repository
                    .find_by_id(command.user_id)
                    .await?
                    .ok_or_else(|| {
                        CoreError::Internal(
                            "accepting user has no local account: auth_middleware should have \
                             created one before this call was ever reachable"
                                .to_owned(),
                        )
                    })?;
                let member = Member {
                    id: MemberId(generate_uuid_v7()),
                    organization_id: invitation.organization_id,
                    user_id: Some(command.user_id),
                    last_name: account.name,
                    first_name: None,
                    joined_at: Some(now),
                    created_at: now,
                    deleted_at: None,
                };
                self.member_repository.insert(&member).await?
            }
        };

        self.invitation_repository
            .mark_consumed(invitation.id, now, command.user_id)
            .await?;

        self.emitter.emit(
            member.organization_id,
            &MemberJoined {
                member: member.clone(),
                invitation_id: invitation.id,
            },
        )?;

        Ok(member)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        UserId,
        domain::{
            invitation::ports::MockInvitationRepository, member::ports::MockMemberRepository,
            role::ports::MockRoleRepository, user::User, user::ports::MockUserRepository,
        },
    };
    use authz::{Decision, MockAuthorizer};
    use chrono::Duration;
    use events::testing::RecordingEmitter;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org_id() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn user_id() -> UserId {
        UserId(Uuid::new_v4())
    }

    fn actor_for(user_id: UserId) -> Subject {
        policy::user_subject(user_id, Vec::new())
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

    fn member(id: MemberId, organization_id: OrganizationId) -> Member {
        let now = Utc::now();
        Member {
            id,
            organization_id,
            user_id: None,
            last_name: "Vacant".to_owned(),
            first_name: None,
            joined_at: None,
            created_at: now,
            deleted_at: None,
        }
    }

    fn invitation(organization_id: OrganizationId, member_id: Option<MemberId>) -> Invitation {
        let now = Utc::now();
        Invitation {
            id: InvitationId(Uuid::new_v4()),
            organization_id,
            member_id,
            token_hash: token::hash("clear-token"),
            expires_at: now + Duration::days(7),
            consumed_at: None,
            consumed_by_user_id: None,
            created_by_user_id: UserId(Uuid::new_v4()),
            created_at: now,
        }
    }

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

    fn allow() -> MockAuthorizer {
        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
        authz
    }

    fn deny() -> MockAuthorizer {
        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::deny()) }));
        authz
    }

    #[allow(clippy::too_many_arguments)]
    fn service(
        invitations: MockInvitationRepository,
        members: MockMemberRepository,
        roles: MockRoleRepository,
        users: MockUserRepository,
        authz: MockAuthorizer,
    ) -> InvitationService<
        MockInvitationRepository,
        MockMemberRepository,
        MockRoleRepository,
        MockUserRepository,
        MockAuthorizer,
        RecordingEmitter,
    > {
        InvitationService::new(
            invitations,
            members,
            roles,
            users,
            authz,
            RecordingEmitter::new(),
        )
    }

    #[tokio::test]
    async fn invite_member_persists_and_emits_when_targeting_no_seat() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);

        let mut invitations = MockInvitationRepository::new();
        invitations.expect_insert().times(1).returning(|i| {
            let cloned = i.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(
            invitations,
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        let (created, clear_token) = service
            .invite_member(InviteMemberCommand {
                actor: actor_for(uid),
                organization_id: oid,
                member_id: None,
                expires_at: Utc::now() + Duration::days(7),
            })
            .await
            .unwrap();

        assert_eq!(created.organization_id, oid);
        assert_eq!(created.member_id, None);
        assert!(created.consumed_at.is_none());
        assert_eq!(clear_token.len(), 64);
        assert_eq!(token::hash(&clear_token), created.token_hash);
    }

    #[tokio::test]
    async fn invite_member_targets_an_existing_vacant_seat() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());
        let target_id = MemberId(Uuid::new_v4());

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);
        members
            .expect_find_by_id()
            .with(eq(target_id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(target_id, oid))) }));

        let mut invitations = MockInvitationRepository::new();
        invitations.expect_insert().times(1).returning(|i| {
            let cloned = i.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(
            invitations,
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        let (created, _token) = service
            .invite_member(InviteMemberCommand {
                actor: actor_for(uid),
                organization_id: oid,
                member_id: Some(target_id),
                expires_at: Utc::now() + Duration::days(7),
            })
            .await
            .unwrap();

        assert_eq!(created.member_id, Some(target_id));
    }

    #[tokio::test]
    async fn invite_member_rejects_a_seat_from_another_organization() {
        let oid = org_id();
        let other_oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());
        let target_id = MemberId(Uuid::new_v4());

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);
        members
            .expect_find_by_id()
            .with(eq(target_id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(target_id, other_oid))) }));
        // No `expect_insert` — must short-circuit before mutation.

        let mut service = service(
            MockInvitationRepository::new(),
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        let err = service
            .invite_member(InviteMemberCommand {
                actor: actor_for(uid),
                organization_id: oid,
                member_id: Some(target_id),
                expires_at: Utc::now() + Duration::days(7),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn invite_member_rejects_an_already_occupied_seat() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());
        let target_id = MemberId(Uuid::new_v4());

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);
        members
            .expect_find_by_id()
            .with(eq(target_id))
            .times(1)
            .returning(move |_| {
                let mut m = member(target_id, oid);
                m.user_id = Some(user_id());
                Box::pin(async move { Ok(Some(m)) })
            });

        let mut service = service(
            MockInvitationRepository::new(),
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        let err = service
            .invite_member(InviteMemberCommand {
                actor: actor_for(uid),
                organization_id: oid,
                member_id: Some(target_id),
                expires_at: Utc::now() + Duration::days(7),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn invite_member_rejects_an_expiry_in_the_past() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);

        let mut service = service(
            MockInvitationRepository::new(),
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        let err = service
            .invite_member(InviteMemberCommand {
                actor: actor_for(uid),
                organization_id: oid,
                member_id: None,
                expires_at: Utc::now() - Duration::days(1),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn invite_member_returns_forbidden_when_authz_denies() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);

        let mut service = service(
            MockInvitationRepository::new(),
            members,
            roles,
            MockUserRepository::new(),
            deny(),
        );

        let err = service
            .invite_member(InviteMemberCommand {
                actor: actor_for(uid),
                organization_id: oid,
                member_id: None,
                expires_at: Utc::now() + Duration::days(7),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn list_pending_invitations_delegates_when_authorized() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_list_pending_by_organization()
            .with(eq(oid))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let mut service = service(
            invitations,
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        let result = service
            .list_pending_invitations(actor_for(uid), oid)
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn revoke_invitation_deletes_a_pending_one() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());
        let inv = invitation(oid, None);
        let inv_id = inv.id;

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_id()
            .with(eq(inv_id))
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });
        invitations
            .expect_revoke()
            .with(eq(inv_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut service = service(
            invitations,
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        service
            .revoke_invitation(RevokeInvitationCommand {
                actor: actor_for(uid),
                invitation_id: inv_id,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn revoke_invitation_rejects_an_already_consumed_one() {
        let oid = org_id();
        let uid = user_id();
        let acting_member_id = MemberId(Uuid::new_v4());
        let mut inv = invitation(oid, None);
        inv.consumed_at = Some(Utc::now());
        let inv_id = inv.id;

        let mut members = MockMemberRepository::new();
        let mut roles = MockRoleRepository::new();
        stage_org_membership(&mut members, &mut roles, oid, uid, acting_member_id);

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_id()
            .with(eq(inv_id))
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });
        // No `expect_revoke` — must short-circuit before mutation.

        let mut service = service(
            invitations,
            members,
            roles,
            MockUserRepository::new(),
            allow(),
        );

        let err = service
            .revoke_invitation(RevokeInvitationCommand {
                actor: actor_for(uid),
                invitation_id: inv_id,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn accept_invitation_creates_a_seat_named_from_the_account_when_no_seat_is_targeted() {
        let oid = org_id();
        let uid = user_id();
        let inv = invitation(oid, None);

        let mut invitations = MockInvitationRepository::new();
        let hash = inv.token_hash.clone();
        invitations
            .expect_find_by_token_hash()
            .withf(move |h| h == hash)
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });
        invitations
            .expect_mark_consumed()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

        let mut members = MockMemberRepository::new();
        members
            .expect_find_by_org_and_user()
            .with(eq(oid), eq(uid))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        members.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut users = MockUserRepository::new();
        users
            .expect_find_by_id()
            .with(eq(uid))
            .times(1)
            .returning(move |id| {
                let u = user(id);
                Box::pin(async move { Ok(Some(u)) })
            });

        let mut service = service(
            invitations,
            members,
            MockRoleRepository::new(),
            users,
            MockAuthorizer::new(),
        );

        let member = service
            .accept_invitation(AcceptInvitationCommand {
                token: "clear-token".to_owned(),
                user_id: uid,
            })
            .await
            .unwrap();

        assert_eq!(member.organization_id, oid);
        assert_eq!(member.user_id, Some(uid));
        assert_eq!(member.last_name, "Alice");
        assert!(member.joined_at.is_some());
    }

    #[tokio::test]
    async fn accept_invitation_occupies_the_targeted_seat() {
        let oid = org_id();
        let uid = user_id();
        let target_id = MemberId(Uuid::new_v4());
        let inv = invitation(oid, Some(target_id));

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_token_hash()
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });
        invitations
            .expect_mark_consumed()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

        let mut members = MockMemberRepository::new();
        members
            .expect_find_by_org_and_user()
            .with(eq(oid), eq(uid))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        members
            .expect_find_by_id()
            .with(eq(target_id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(member(target_id, oid))) }));
        members.expect_update().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(
            invitations,
            members,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let member = service
            .accept_invitation(AcceptInvitationCommand {
                token: "clear-token".to_owned(),
                user_id: uid,
            })
            .await
            .unwrap();

        assert_eq!(member.id, target_id);
        assert_eq!(member.user_id, Some(uid));
    }

    #[tokio::test]
    async fn accept_invitation_rejects_an_unknown_token() {
        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_token_hash()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = service(
            invitations,
            MockMemberRepository::new(),
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let err = service
            .accept_invitation(AcceptInvitationCommand {
                token: "unknown".to_owned(),
                user_id: user_id(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn accept_invitation_rejects_an_expired_token_as_not_found() {
        let oid = org_id();
        let mut inv = invitation(oid, None);
        inv.expires_at = Utc::now() - Duration::minutes(1);

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_token_hash()
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });

        let mut service = service(
            invitations,
            MockMemberRepository::new(),
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let err = service
            .accept_invitation(AcceptInvitationCommand {
                token: "clear-token".to_owned(),
                user_id: user_id(),
            })
            .await
            .unwrap_err();

        // Same variant as an unknown token — see the method's doc comment.
        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn accept_invitation_rejects_an_already_consumed_token_as_not_found() {
        let oid = org_id();
        let mut inv = invitation(oid, None);
        inv.consumed_at = Some(Utc::now());

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_token_hash()
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });

        let mut service = service(
            invitations,
            MockMemberRepository::new(),
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let err = service
            .accept_invitation(AcceptInvitationCommand {
                token: "clear-token".to_owned(),
                user_id: user_id(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn accept_invitation_rejects_a_caller_already_a_member() {
        let oid = org_id();
        let uid = user_id();
        let inv = invitation(oid, None);

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_token_hash()
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });
        // No `expect_mark_consumed` — must short-circuit before mutation.

        let mut members = MockMemberRepository::new();
        members
            .expect_find_by_org_and_user()
            .with(eq(oid), eq(uid))
            .times(1)
            .returning(move |organization_id, user_id| {
                let mut existing = member(MemberId(Uuid::new_v4()), organization_id);
                existing.user_id = Some(user_id);
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(
            invitations,
            members,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let err = service
            .accept_invitation(AcceptInvitationCommand {
                token: "clear-token".to_owned(),
                user_id: uid,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn accept_invitation_rejects_a_seat_occupied_by_a_race() {
        let oid = org_id();
        let uid = user_id();
        let target_id = MemberId(Uuid::new_v4());
        let inv = invitation(oid, Some(target_id));

        let mut invitations = MockInvitationRepository::new();
        invitations
            .expect_find_by_token_hash()
            .times(1)
            .returning(move |_| {
                let cloned = inv.clone();
                Box::pin(async move { Ok(Some(cloned)) })
            });
        // No `expect_mark_consumed` — must short-circuit before mutation.

        let mut members = MockMemberRepository::new();
        members
            .expect_find_by_org_and_user()
            .with(eq(oid), eq(uid))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        members
            .expect_find_by_id()
            .with(eq(target_id))
            .times(1)
            .returning(move |_| {
                let mut occupied = member(target_id, oid);
                occupied.user_id = Some(user_id());
                Box::pin(async move { Ok(Some(occupied)) })
            });

        let mut service = service(
            invitations,
            members,
            MockRoleRepository::new(),
            MockUserRepository::new(),
            MockAuthorizer::new(),
        );

        let err = service
            .accept_invitation(AcceptInvitationCommand {
                token: "clear-token".to_owned(),
                user_id: uid,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }
}
