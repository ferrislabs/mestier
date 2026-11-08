use std::collections::HashMap;
use std::future::Future;

use common::{CoreError, OrganizationId, RoleId, UserId};

use crate::CategoryId;

/// Who may see the channels of a private category: the roles listed, and the
/// individual members listed (the creator is always one, so a category cannot
/// shut its own maker out).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CategoryAccess {
    pub roles: Vec<RoleId>,
    pub users: Vec<UserId>,
}

impl CategoryAccess {
    pub fn admits(&self, user: UserId, user_roles: &[RoleId]) -> bool {
        self.users.contains(&user) || user_roles.iter().any(|role| self.roles.contains(role))
    }
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CategoryAccessRepository: Send {
    /// Makes the category private to exactly `access`.
    fn replace(
        &mut self,
        organization_id: OrganizationId,
        category_id: CategoryId,
        access: &CategoryAccess,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// `None` when the category is open to everyone.
    fn find(
        &mut self,
        category_id: CategoryId,
    ) -> impl Future<Output = Result<Option<CategoryAccess>, CoreError>> + Send;

    /// The access list of every private category of the organization.
    fn list_for_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<HashMap<CategoryId, CategoryAccess>, CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn role(n: u128) -> RoleId {
        RoleId(Uuid::from_u128(n))
    }

    fn user(n: u128) -> UserId {
        UserId(Uuid::from_u128(n))
    }

    #[test]
    fn a_listed_role_admits_its_holders() {
        let access = CategoryAccess {
            roles: vec![role(1)],
            users: vec![],
        };

        assert!(access.admits(user(9), &[role(1), role(2)]));
        assert!(!access.admits(user(9), &[role(2)]));
    }

    #[test]
    fn a_listed_member_is_admitted_whatever_their_roles() {
        let access = CategoryAccess {
            roles: vec![role(1)],
            users: vec![user(9)],
        };

        assert!(access.admits(user(9), &[]));
        assert!(!access.admits(user(8), &[]));
    }
}
