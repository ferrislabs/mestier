use axum_extra::routing::TypedPath;
use mestier_core::{MemberId, OrganizationId};
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
