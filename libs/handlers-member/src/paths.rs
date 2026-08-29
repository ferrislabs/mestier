use axum_extra::routing::TypedPath;
use mestier_core::{InvitationId, MemberId, OrganizationId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members")]
pub struct MembersPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/members/{member_id}")]
pub struct MemberPath {
    pub member_id: MemberId,
}

/// The caller's own aggregated permissions in one organization (#307) —
/// `me`, never a member id, because a caller must never be able to read
/// anyone else's bits through this route by supplying a different id.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/me/permissions")]
pub struct MyPermissionsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/invitations")]
pub struct InvitationsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invitations/{invitation_id}")]
pub struct InvitationPath {
    pub invitation_id: InvitationId,
}

/// `token` is a bare `String`, not a typed id: it identifies a request, not
/// a stored resource — see `AcceptInvitationCommand`'s doc comment.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/invitations/{token}/accept")]
pub struct AcceptInvitationPath {
    pub token: String,
}
