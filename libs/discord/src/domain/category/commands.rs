use common::OrganizationId;

use crate::CategoryId;

pub struct CreateCategoryCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub position: i32,
}

pub struct UpdateCategoryCommand {
    pub id: CategoryId,
    pub name: String,
    pub position: i32,
}
