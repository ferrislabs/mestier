use authz::Subject;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Member,
    application::MestierUseCase,
    domain::{
        invitation::{
            Invitation,
            commands::{AcceptInvitationCommand, InviteMemberCommand, RevokeInvitationCommand},
            service::InvitationService,
        },
        organization::OrganizationId,
    },
};

impl MestierUseCase {
    /// Returns the invitation together with its clear token — see
    /// `InvitationService::invite_member`'s doc comment for why that is the
    /// one and only time it is ever readable again.
    #[transactional(invitation, member, role, user, authz, emitter)]
    pub async fn invite_member(
        &self,
        command: InviteMemberCommand,
    ) -> Result<(Invitation, String), CoreError> {
        let mut service = InvitationService::new(
            invitation_repository,
            member_repository,
            role_repository,
            user_repository,
            authz,
            emitter,
        );
        service.invite_member(command).await
    }

    #[transactional(invitation, member, role, user, authz, emitter)]
    pub async fn list_pending_invitations(
        &self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<Vec<Invitation>, CoreError> {
        let mut service = InvitationService::new(
            invitation_repository,
            member_repository,
            role_repository,
            user_repository,
            authz,
            emitter,
        );
        service
            .list_pending_invitations(actor, organization_id)
            .await
    }

    #[transactional(invitation, member, role, user, authz, emitter)]
    pub async fn revoke_invitation(
        &self,
        command: RevokeInvitationCommand,
    ) -> Result<(), CoreError> {
        let mut service = InvitationService::new(
            invitation_repository,
            member_repository,
            role_repository,
            user_repository,
            authz,
            emitter,
        );
        service.revoke_invitation(command).await
    }

    /// No `Subject`/authorization parameter — see
    /// `AcceptInvitationCommand`'s doc comment: this is the one route in the
    /// whole chantier that must succeed for a caller with no standing in the
    /// target organization.
    #[transactional(invitation, member, role, user, authz, emitter)]
    pub async fn accept_invitation(
        &self,
        command: AcceptInvitationCommand,
    ) -> Result<Member, CoreError> {
        let mut service = InvitationService::new(
            invitation_repository,
            member_repository,
            role_repository,
            user_repository,
            authz,
            emitter,
        );
        service.accept_invitation(command).await
    }
}
