use std::collections::HashSet;

use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::CoreError;

use super::{
    MemberProfile, MemberProfileError, MemberProfileOutcome,
    commands::{
        ArchiveMemberCommand, GetMemberProfileCommand, ListMemberProfilesCommand,
        PutMemberProfileCommand, RestoreMemberCommand,
    },
    ports::MemberProfileRepository,
};
use crate::{
    MemberId, OrganizationId,
    application::policy,
    domain::{
        member::{Member, ports::MemberRepository},
        role::ports::RoleRepository,
    },
};

pub struct MemberProfileService<P, M, Ro, A>
where
    P: MemberProfileRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    profiles: P,
    members: M,
    roles: Ro,
    authz: A,
}

impl<P, M, Ro, A> MemberProfileService<P, M, Ro, A>
where
    P: MemberProfileRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(profiles: P, members: M, roles: Ro, authz: A) -> Self {
        Self {
            profiles,
            members,
            roles,
            authz,
        }
    }

    pub async fn get_profile(
        &mut self,
        command: GetMemberProfileCommand,
    ) -> Result<MemberProfile, CoreError> {
        self.enrich(command.actor, command.organization_id).await?;
        self.load_or_empty(command.organization_id, command.member_id)
            .await
    }

    pub async fn list_profiles(
        &mut self,
        command: ListMemberProfilesCommand,
    ) -> Result<Vec<MemberProfile>, CoreError> {
        self.enrich(command.actor, command.organization_id).await?;
        self.profiles
            .list_by_organization(command.organization_id)
            .await
    }

    pub async fn put_profile(
        &mut self,
        command: PutMemberProfileCommand,
    ) -> Result<MemberProfileOutcome<MemberProfile>, CoreError> {
        let actor = self.enrich(command.actor, command.organization_id).await?;
        self.authorize_write(&actor, command.organization_id)
            .await?;
        let current = self
            .load_or_empty(command.organization_id, command.member_id)
            .await?;

        let replaced = match current.replaced_with(command.input, Utc::now()) {
            Ok(replaced) => replaced,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if let Some(manager) = replaced.manager_member_id
            && let Some(refusal) = self
                .manager_refusal(command.organization_id, command.member_id, manager)
                .await?
        {
            return Ok(Err(refusal));
        }

        self.profiles.upsert(&replaced).await.map(Ok)
    }

    pub async fn archive(
        &mut self,
        command: ArchiveMemberCommand,
    ) -> Result<MemberProfile, CoreError> {
        let actor = self.enrich(command.actor, command.organization_id).await?;
        self.authorize_write(&actor, command.organization_id)
            .await?;
        let current = self
            .load_or_empty(command.organization_id, command.member_id)
            .await?;

        self.profiles.upsert(&current.archived(Utc::now())).await
    }

    pub async fn restore(
        &mut self,
        command: RestoreMemberCommand,
    ) -> Result<MemberProfile, CoreError> {
        let actor = self.enrich(command.actor, command.organization_id).await?;
        self.authorize_write(&actor, command.organization_id)
            .await?;
        let current = self
            .load_or_empty(command.organization_id, command.member_id)
            .await?;

        self.profiles.upsert(&current.restored(Utc::now())).await
    }

    async fn enrich(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<Subject, CoreError> {
        policy::enrich_for_organization(actor, organization_id, &mut self.members, &mut self.roles)
            .await
    }

    async fn authorize_write(
        &self,
        actor: &Subject,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        policy::require(
            &self.authz,
            actor,
            "member.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await
    }

    async fn member_of(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<Member, CoreError> {
        self.members
            .find_by_id(member_id)
            .await?
            .filter(|member| member.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    async fn load_or_empty(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<MemberProfile, CoreError> {
        self.member_of(organization_id, member_id).await?;
        Ok(self
            .profiles
            .find_by_member(member_id)
            .await?
            .unwrap_or_else(|| MemberProfile::empty(member_id, organization_id, Utc::now())))
    }

    async fn manager_refusal(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        manager: MemberId,
    ) -> Result<Option<MemberProfileError>, CoreError> {
        if manager == member_id {
            return Ok(Some(MemberProfileError::ManagerIsSelf));
        }
        let in_organization = self
            .members
            .find_by_id(manager)
            .await?
            .is_some_and(|found| {
                found.organization_id == organization_id && found.deleted_at.is_none()
            });
        if !in_organization {
            return Ok(Some(MemberProfileError::ManagerNotInOrganization));
        }

        let mut visited = HashSet::new();
        let mut cursor = manager;
        while visited.insert(cursor) {
            let Some(next) = self
                .profiles
                .find_by_member(cursor)
                .await?
                .and_then(|profile| profile.manager_member_id)
            else {
                break;
            };
            if next == member_id {
                return Ok(Some(MemberProfileError::ManagerCycle));
            }
            cursor = next;
        }
        Ok(None)
    }
}
