use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};

use super::{
    DayOverride, DayOverrideId, DayOverrideInput, DayOverrideOutcome, PlanningPeriod,
    ports::DayOverrideRepository,
};
use crate::{MemberId, OrganizationId, domain::member::ports::MemberRepository};

pub struct DayOverrideService<R, M>
where
    R: DayOverrideRepository,
    M: MemberRepository,
{
    overrides: R,
    members: M,
}

impl<R, M> DayOverrideService<R, M>
where
    R: DayOverrideRepository,
    M: MemberRepository,
{
    pub fn new(overrides: R, members: M) -> Self {
        Self { overrides, members }
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        period: PlanningPeriod,
        member_id: Option<MemberId>,
    ) -> Result<Vec<DayOverride>, CoreError> {
        self.overrides
            .list(organization_id, period, member_id)
            .await
    }

    pub async fn put(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        date: NaiveDate,
        input: &DayOverrideInput,
    ) -> Result<DayOverrideOutcome<DayOverride>, CoreError> {
        self.require_member(organization_id, member_id).await?;
        let day_override = match DayOverride::declare(
            DayOverrideId(generate_uuid_v7()),
            organization_id,
            member_id,
            date,
            input,
            Utc::now(),
        ) {
            Ok(day_override) => day_override,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.overrides.upsert(&day_override).await.map(Ok)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        date: NaiveDate,
    ) -> Result<(), CoreError> {
        self.require_member(organization_id, member_id).await?;
        if self
            .overrides
            .delete(organization_id, member_id, date)
            .await?
        {
            Ok(())
        } else {
            Err(CoreError::NotFound)
        }
    }

    async fn require_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<(), CoreError> {
        self.members
            .find_by_id(member_id)
            .await?
            .filter(|member| member.organization_id == organization_id)
            .map(|_| ())
            .ok_or(CoreError::NotFound)
    }
}
