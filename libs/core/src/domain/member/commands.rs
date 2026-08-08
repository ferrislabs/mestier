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
    pub organization_id: OrganizationId,
    pub last_name: String,
    /// `None` = not provided, distinct from an empty string.
    pub first_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateMemberCommand {
    pub member_id: MemberId,
    pub last_name: String,
    /// `None` = not provided, distinct from an empty string.
    pub first_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssignRoleCommand {
    pub member_id: MemberId,
    pub role_id: RoleId,
}
