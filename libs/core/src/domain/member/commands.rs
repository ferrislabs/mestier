use crate::{
    UserId,
    domain::{member::MemberId, organization::OrganizationId, role::RoleId},
};

#[derive(Debug, Clone)]
pub struct AddMemberCommand {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct AssignRoleCommand {
    pub member_id: MemberId,
    pub role_id: RoleId,
}
