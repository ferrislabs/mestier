use authz::Subject;
use chrono::{DateTime, Utc};

use crate::{
    UserId,
    domain::{invitation::InvitationId, member::MemberId, organization::OrganizationId},
};

#[derive(Debug, Clone)]
pub struct InviteMemberCommand {
    /// Authenticated actor performing the invite. Built by the handler from
    /// the request `Identity`, mirrors `CreateMemberCommand::actor`.
    pub actor: Subject,
    pub organization_id: OrganizationId,
    /// `Some` — grant login access to an existing, vacant seat. `None` —
    /// nobody has a seat yet; acceptance creates one.
    pub member_id: Option<MemberId>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RevokeInvitationCommand {
    pub actor: Subject,
    pub invitation_id: InvitationId,
}

/// No `actor: Subject` — deliberately. Accepting an invitation is the one
/// use case in this module that must succeed for a caller with *no* standing
/// in the target organization; the standard `enrich_for_organization` +
/// `policy::require` shape that every other command here goes through would
/// reject exactly the request this one exists to serve.
///
/// `user_id` is the caller's local id, already resolved by the handler from
/// the authenticated `Identity` — the same resolution `create_member`'s
/// handler does before authorization even starts.
#[derive(Debug, Clone)]
pub struct AcceptInvitationCommand {
    pub token: String,
    pub user_id: UserId,
}
