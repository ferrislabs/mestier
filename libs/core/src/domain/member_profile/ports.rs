use common::CoreError;

use crate::{MemberId, MemberProfile};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MemberProfileRepository: Send {
    fn find_by_member(
        &mut self,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Option<MemberProfile>, CoreError>> + Send;

    fn upsert(
        &mut self,
        profile: &MemberProfile,
    ) -> impl Future<Output = Result<MemberProfile, CoreError>> + Send;
}
