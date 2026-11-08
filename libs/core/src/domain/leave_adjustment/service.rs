use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    LeaveAdjustment, LeaveAdjustmentError, LeaveAdjustmentId, LeaveAdjustmentOutcome,
    commands::LeaveAdjustmentDraft, ports::LeaveAdjustmentRepository,
};
use crate::{
    LeaveAccountId, MemberId, OrganizationId,
    domain::{leave_account::ports::LeaveAccountRepository, member::ports::MemberRepository},
};

pub struct LeaveAdjustmentService<R, A, M>
where
    R: LeaveAdjustmentRepository,
    A: LeaveAccountRepository,
    M: MemberRepository,
{
    adjustments: R,
    accounts: A,
    members: M,
}

impl<R, A, M> LeaveAdjustmentService<R, A, M>
where
    R: LeaveAdjustmentRepository,
    A: LeaveAccountRepository,
    M: MemberRepository,
{
    pub fn new(adjustments: R, accounts: A, members: M) -> Self {
        Self {
            adjustments,
            accounts,
            members,
        }
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        member_id: Option<MemberId>,
        account_id: Option<LeaveAccountId>,
    ) -> Result<Vec<LeaveAdjustment>, CoreError> {
        self.adjustments
            .list(organization_id, member_id, account_id)
            .await
    }

    pub async fn create(
        &mut self,
        organization_id: OrganizationId,
        draft: &LeaveAdjustmentDraft,
        author_member_id: Option<MemberId>,
    ) -> Result<LeaveAdjustmentOutcome<LeaveAdjustment>, CoreError> {
        let Some(account) = self
            .accounts
            .find(organization_id, draft.account_id)
            .await?
        else {
            return Ok(Err(LeaveAdjustmentError::AccountUnknown));
        };
        let known_member = self
            .members
            .find_by_id(draft.member_id)
            .await?
            .is_some_and(|member| member.organization_id == organization_id);
        if !known_member {
            return Ok(Err(LeaveAdjustmentError::MemberUnknown));
        }
        let adjustment = match LeaveAdjustment::record(
            LeaveAdjustmentId(generate_uuid_v7()),
            organization_id,
            draft,
            &account,
            author_member_id,
            Utc::now(),
        ) {
            Ok(adjustment) => adjustment,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.adjustments.insert(&adjustment).await.map(Ok)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAdjustmentId,
    ) -> Result<(), CoreError> {
        self.adjustments
            .find(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        self.adjustments.delete(organization_id, id).await
    }
}
