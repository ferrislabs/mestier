use common::CoreError;
use mestier_macros::transactional;

use crate::{
    UserId,
    application::MestierUseCase,
    domain::{member::ports::MemberRepository, organization::OrganizationId},
};

/// One seated member as chat sees them: the `users.id` that message authors,
/// reactions and presence are keyed by, and the name to show for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatDirectoryEntry {
    pub user_id: UserId,
    pub display_name: String,
}

impl MestierUseCase {
    /// Active members that someone occupies. A vacant seat has no `user_id`
    /// and so can neither write nor be present — it has no place here.
    #[transactional(member)]
    pub async fn list_chat_directory(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<ChatDirectoryEntry>, CoreError> {
        let mut member_repository = member_repository;
        let members = member_repository
            .list_active_by_organization(organization_id)
            .await?;
        Ok(members
            .into_iter()
            .filter_map(|member| {
                let display_name = member.display_name();
                member.user_id.map(|user_id| ChatDirectoryEntry {
                    user_id,
                    display_name,
                })
            })
            .collect())
    }
}
