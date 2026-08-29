use authz::Subject;

use crate::{
    UserId,
    domain::{member::MemberId, organization::OrganizationId, role::RoleId},
};

#[derive(Debug, Clone)]
pub struct AddMemberCommand {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub last_name: String,
    /// `None` = not provided, distinct from an empty string.
    pub first_name: Option<String>,
}

/// A free, named seat: `user_id` is not part of this command — it stays
/// `None` until an invitation (#184) fills it.
#[derive(Debug, Clone)]
pub struct CreateMemberCommand {
    /// Authenticated actor performing the creation. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject the
    /// policy engine consumes.
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub last_name: String,
    /// `None` = not provided, distinct from an empty string.
    pub first_name: Option<String>,
}

/// Carries a partial update, not final values: the read-modify-write that
/// used to happen in the handler (loading the member to authorize, then
/// falling absent fields back to what is already stored) now happens in
/// `MemberService::update_member`, since the handler no longer loads the
/// member itself.
#[derive(Debug, Clone)]
pub struct UpdateMemberCommand {
    /// Authenticated actor performing the update.
    pub actor: Subject,
    pub member_id: MemberId,
    /// `None` leaves the stored value untouched.
    pub last_name: Option<String>,
    /// Outer `None` leaves it untouched; `Some(None)` clears it.
    pub first_name: Option<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct AssignRoleCommand {
    /// Authenticated actor performing the assignment.
    pub actor: Subject,
    pub member_id: MemberId,
    pub role_id: RoleId,
}
