use common::CoreError;

use crate::{OrganizationId, Presence, UserId};

#[cfg_attr(test, mockall::automock)]
pub trait PresenceRepository: Send {
    fn upsert(&mut self, p: &Presence) -> impl Future<Output = Result<Presence, CoreError>> + Send;

    fn find(
        &mut self,
        org: OrganizationId,
        user: UserId,
    ) -> impl Future<Output = Result<Option<Presence>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        org: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Presence>, CoreError>> + Send;
}
