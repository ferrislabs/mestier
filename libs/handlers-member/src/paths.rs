use axum_extra::routing::TypedPath;
use mestier_core::{InvitationId, MemberId, OrganizationId, RoleId};
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/roles")]
pub struct RolesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/roles/{role_id}")]
pub struct RolePath {
    pub role_id: RoleId,
}

/// Which roles a member holds, and assigning one — #308. Nested under the
/// member rather than the role: a role can be listed once and assigned to
/// many members, but this is always read/written from one member's side.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/members/{member_id}/roles")]
pub struct MemberRolesPath {
    pub member_id: MemberId,
}

/// One role held by one member — the unassign target. `role_id` sits in the
/// path rather than the body, unlike `POST .../roles`: a `DELETE` with a body
/// is the odd one out, and `role::delete::handler`'s own
/// `DELETE /api/v1/roles/{role_id}` already sets that precedent.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/members/{member_id}/roles/{role_id}")]
pub struct MemberRolePath {
    pub member_id: MemberId,
    pub role_id: RoleId,
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
