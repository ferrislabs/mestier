use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    ExtraPeriod, ExtraPeriodError, ExtraPeriodId, ExtraPeriodOutcome, ExtraStatus,
    commands::{ExtraPeriodDraft, ExtraPeriodFilter},
    ports::ExtraPeriodRepository,
};
use crate::{
    MemberId, OrganizationId,
    domain::{member::ports::MemberRepository, surcharge::ports::SurchargeRepository},
};

pub struct ExtraPeriodService<R, S, M>
where
    R: ExtraPeriodRepository,
    S: SurchargeRepository,
    M: MemberRepository,
{
    periods: R,
    surcharges: S,
    members: M,
}

impl<R, S, M> ExtraPeriodService<R, S, M>
where
    R: ExtraPeriodRepository,
    S: SurchargeRepository,
    M: MemberRepository,
{
    pub fn new(periods: R, surcharges: S, members: M) -> Self {
        Self {
            periods,
            surcharges,
            members,
        }
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &ExtraPeriodFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ExtraPeriod>, u64), CoreError> {
        self.periods
            .list(organization_id, filter, limit, offset)
            .await
    }

    pub async fn create(
        &mut self,
        organization_id: OrganizationId,
        draft: &ExtraPeriodDraft,
        creator: Option<MemberId>,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let known_member = self
            .members
            .find_by_id(draft.member_id)
            .await?
            .is_some_and(|member| member.organization_id == organization_id);
        if !known_member {
            return Ok(Err(ExtraPeriodError::MemberUnknown));
        }
        let surcharge = match draft.surcharge_id {
            Some(id) => match self.surcharges.find(organization_id, id).await? {
                Some(surcharge) => Some(surcharge),
                None => return Ok(Err(ExtraPeriodError::SurchargeUnknown)),
            },
            None => None,
        };
        let period = match ExtraPeriod::declare(
            ExtraPeriodId(generate_uuid_v7()),
            organization_id,
            draft,
            surcharge.as_ref(),
            creator,
            Utc::now(),
        ) {
            Ok(period) => period,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.periods.insert(&period).await.map(Ok)
    }

    pub async fn approve(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
        decider: Option<MemberId>,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut period = self.find(organization_id, id).await?;
        if let Err(refusal) = period.approve(decider, Utc::now()) {
            return Ok(Err(refusal));
        }
        self.periods.update(&period).await.map(Ok)
    }

    pub async fn reject(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
        comment: &str,
        decider: Option<MemberId>,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut period = self.find(organization_id, id).await?;
        if let Err(refusal) = period.reject(comment, decider, Utc::now()) {
            return Ok(Err(refusal));
        }
        self.periods.update(&period).await.map(Ok)
    }

    pub async fn cancel(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
        caller: Option<MemberId>,
        privileged: bool,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut period = self.find(organization_id, id).await?;
        let is_creator = caller.is_some_and(|member| {
            period.created_by_member_id == Some(member) && period.member_id == member
        });
        if !privileged && !is_creator {
            return Err(CoreError::Forbidden {
                reason: Some("only the creator or a planning manager can cancel".to_owned()),
            });
        }
        if !privileged && period.status != ExtraStatus::Pending {
            return Ok(Err(ExtraPeriodError::AlreadyDecided));
        }
        if let Err(refusal) = period.cancel(caller, Utc::now()) {
            return Ok(Err(refusal));
        }
        self.periods.update(&period).await.map(Ok)
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
    ) -> Result<ExtraPeriod, CoreError> {
        self.periods
            .find(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
    }
}
