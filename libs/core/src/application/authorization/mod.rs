use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{UserId, application::MestierUseCase, domain::organization::OrganizationId};

impl MestierUseCase {
    /// Authorizes `action` for `user_id` in the given `organization_id` by
    /// delegating entirely to the canonical `LocalPolicyEngine` (`self.authz`).
    ///
    /// This is the **single source of truth** for permission checks: it calls
    /// `policy::enrich_for_organization` (which aggregates role bits via the
    /// real repositories) and then `policy::require` (which evaluates the
    /// configured action → bit map). External crates must call this instead
    /// of re-implementing the bit-aggregation logic.
    #[transactional(member, role, authz)]
    pub async fn authorize_action(
        &self,
        user_id: UserId,
        iam_roles: Vec<String>,
        organization_id: OrganizationId,
        action: &str,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let subject = crate::application::policy::user_subject(user_id, iam_roles);
        let enriched = crate::application::policy::enrich_for_organization(
            subject,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        let resource = Resource::new("discord", organization_id.to_string());
        crate::application::policy::require(authz, &enriched, action, resource).await
    }
}
