use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use super::{
    ExtraPeriod, ExtraPeriodError, ExtraPeriodId, ExtraPeriodOutcome, ExtraStatus,
    commands::{ExtraPeriodDraft, ExtraPeriodFilter},
    events::{ExtraPeriodDecided, ExtraPeriodRequested},
    ports::ExtraPeriodRepository,
};
use crate::{
    MemberId, OrganizationId,
    domain::{member::ports::MemberRepository, surcharge::ports::SurchargeRepository},
};

pub struct ExtraPeriodService<R, S, M, E>
where
    R: ExtraPeriodRepository,
    S: SurchargeRepository,
    M: MemberRepository,
    E: EventEmitter,
{
    periods: R,
    surcharges: S,
    members: M,
    emitter: E,
}

impl<R, S, M, E> ExtraPeriodService<R, S, M, E>
where
    R: ExtraPeriodRepository,
    S: SurchargeRepository,
    M: MemberRepository,
    E: EventEmitter,
{
    pub fn new(periods: R, surcharges: S, members: M, emitter: E) -> Self {
        Self {
            periods,
            surcharges,
            members,
            emitter,
        }
    }

    fn emit_decision(&self, period: &ExtraPeriod, from: ExtraStatus) -> Result<(), CoreError> {
        self.emitter.emit(
            period.organization_id,
            &ExtraPeriodDecided {
                period: period.clone(),
                from,
            },
        )
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
        let created = self.periods.insert(&period).await?;

        self.emitter.emit(
            created.organization_id,
            &ExtraPeriodRequested {
                period: created.clone(),
            },
        )?;
        if created.status == ExtraStatus::Approved {
            self.emit_decision(&created, ExtraStatus::Pending)?;
        }

        Ok(Ok(created))
    }

    pub async fn approve(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
        decider: Option<MemberId>,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut period = self.find(organization_id, id).await?;
        let from = period.status;
        if let Err(refusal) = period.approve(decider, Utc::now()) {
            return Ok(Err(refusal));
        }
        let saved = self.periods.update(&period).await?;
        self.emit_decision(&saved, from)?;
        Ok(Ok(saved))
    }

    pub async fn reject(
        &mut self,
        organization_id: OrganizationId,
        id: ExtraPeriodId,
        comment: &str,
        decider: Option<MemberId>,
    ) -> Result<ExtraPeriodOutcome<ExtraPeriod>, CoreError> {
        let mut period = self.find(organization_id, id).await?;
        let from = period.status;
        if let Err(refusal) = period.reject(comment, decider, Utc::now()) {
            return Ok(Err(refusal));
        }
        let saved = self.periods.update(&period).await?;
        self.emit_decision(&saved, from)?;
        Ok(Ok(saved))
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
        let from = period.status;
        if let Err(refusal) = period.cancel(caller, Utc::now()) {
            return Ok(Err(refusal));
        }
        let saved = self.periods.update(&period).await?;
        self.emit_decision(&saved, from)?;
        Ok(Ok(saved))
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use events::testing::RecordingEmitter;
    use uuid::Uuid;

    use super::*;
    use crate::Member;
    use crate::domain::{
        extra_period::ports::MockExtraPeriodRepository, member::ports::MockMemberRepository,
        surcharge::ports::MockSurchargeRepository,
    };

    type Service<'a> = ExtraPeriodService<
        MockExtraPeriodRepository,
        MockSurchargeRepository,
        MockMemberRepository,
        &'a RecordingEmitter,
    >;

    fn member_of(organization_id: OrganizationId) -> Member {
        Member {
            id: MemberId(Uuid::new_v4()),
            organization_id,
            user_id: None,
            last_name: "Martin".to_owned(),
            first_name: None,
            joined_at: None,
            created_at: Utc::now(),
            deleted_at: None,
        }
    }

    fn members_holding(member: Member) -> MockMemberRepository {
        let mut members = MockMemberRepository::new();
        members.expect_find_by_id().returning(move |_| {
            let found = member.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        members
    }

    fn draft(member_id: MemberId, approve: bool) -> ExtraPeriodDraft {
        ExtraPeriodDraft {
            member_id,
            date: NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
            start: "08:00".to_owned(),
            end: "12:00".to_owned(),
            lunch: None,
            reason: "Rattrapage du chantier".to_owned(),
            surcharge_id: None,
            rate_bp: None,
            approve,
        }
    }

    fn pending(organization_id: OrganizationId) -> ExtraPeriod {
        ExtraPeriod::declare(
            ExtraPeriodId(generate_uuid_v7()),
            organization_id,
            &draft(MemberId(Uuid::new_v4()), false),
            None,
            None,
            Utc::now(),
        )
        .unwrap()
    }

    fn echo(repository: &mut MockExtraPeriodRepository) {
        repository.expect_insert().returning(|p| {
            let cloned = p.clone();
            Box::pin(async move { Ok(cloned) })
        });
        repository.expect_update().returning(|p| {
            let cloned = p.clone();
            Box::pin(async move { Ok(cloned) })
        });
    }

    fn holding(existing: ExtraPeriod) -> MockExtraPeriodRepository {
        let mut repository = MockExtraPeriodRepository::new();
        repository.expect_find().returning(move |_, _| {
            let found = existing.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        echo(&mut repository);
        repository
    }

    fn service(
        repository: MockExtraPeriodRepository,
        members: MockMemberRepository,
        emitter: &RecordingEmitter,
    ) -> Service<'_> {
        ExtraPeriodService::new(repository, MockSurchargeRepository::new(), members, emitter)
    }

    #[tokio::test]
    async fn declaring_emits_requested_with_the_pending_period() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let member = member_of(organization_id);
        let member_id = member.id;
        let mut repository = MockExtraPeriodRepository::new();
        echo(&mut repository);
        let emitter = RecordingEmitter::new();
        let mut service = service(repository, members_holding(member), &emitter);

        service
            .create(organization_id, &draft(member_id, false), None)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(emitter.names(), vec!["extra_period.requested"]);
        let event = emitter.only("extra_period.requested");
        assert_eq!(event.payload["extra_period"]["status"], "PENDING");
        assert_eq!(event.payload["extra_period"]["start"], "08:00");
        assert_eq!(event.payload["extra_period"]["worked_minutes"], 240);
    }

    #[tokio::test]
    async fn declaring_already_approved_emits_requested_then_approved() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let member = member_of(organization_id);
        let member_id = member.id;
        let mut repository = MockExtraPeriodRepository::new();
        echo(&mut repository);
        let emitter = RecordingEmitter::new();
        let mut service = service(repository, members_holding(member), &emitter);

        service
            .create(organization_id, &draft(member_id, true), None)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            emitter.names(),
            vec!["extra_period.requested", "extra_period.approved"]
        );
        assert_eq!(
            emitter.only("extra_period.approved").payload["from"],
            "PENDING"
        );
    }

    #[tokio::test]
    async fn a_refused_declaration_emits_nothing() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut members = MockMemberRepository::new();
        members
            .expect_find_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));
        let emitter = RecordingEmitter::new();
        let mut service = service(MockExtraPeriodRepository::new(), members, &emitter);

        let outcome = service
            .create(
                organization_id,
                &draft(MemberId(Uuid::new_v4()), false),
                None,
            )
            .await
            .unwrap();

        assert_eq!(outcome, Err(ExtraPeriodError::MemberUnknown));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn a_failed_insert_emits_nothing() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let member = member_of(organization_id);
        let member_id = member.id;
        let mut repository = MockExtraPeriodRepository::new();
        repository
            .expect_insert()
            .returning(|_| Box::pin(async { Err(CoreError::Conflict("boom".to_owned())) }));
        let emitter = RecordingEmitter::new();
        let mut service = service(repository, members_holding(member), &emitter);

        let result = service
            .create(organization_id, &draft(member_id, false), None)
            .await;

        assert!(result.is_err());
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn approving_emits_approved_from_pending() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = pending(organization_id);
        let id = existing.id;
        let emitter = RecordingEmitter::new();
        let mut service = service(holding(existing), MockMemberRepository::new(), &emitter);

        service
            .approve(organization_id, id, None)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(emitter.names(), vec!["extra_period.approved"]);
        let event = emitter.only("extra_period.approved");
        assert_eq!(event.payload["from"], "PENDING");
        assert_eq!(event.payload["to"], "APPROVED");
    }

    #[tokio::test]
    async fn rejecting_emits_rejected_with_the_decision_comment() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = pending(organization_id);
        let id = existing.id;
        let emitter = RecordingEmitter::new();
        let mut service = service(holding(existing), MockMemberRepository::new(), &emitter);

        service
            .reject(organization_id, id, "Non validé", None)
            .await
            .unwrap()
            .unwrap();

        let event = emitter.only("extra_period.rejected");
        assert_eq!(event.payload["from"], "PENDING");
        assert_eq!(
            event.payload["extra_period"]["decision_comment"],
            "Non validé"
        );
    }

    #[tokio::test]
    async fn cancelling_emits_cancelled() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = pending(organization_id);
        let id = existing.id;
        let emitter = RecordingEmitter::new();
        let mut service = service(holding(existing), MockMemberRepository::new(), &emitter);

        service
            .cancel(organization_id, id, None, true)
            .await
            .unwrap()
            .unwrap();

        let event = emitter.only("extra_period.cancelled");
        assert_eq!(event.payload["from"], "PENDING");
        assert_eq!(event.payload["to"], "CANCELLED");
    }

    #[tokio::test]
    async fn a_refused_decision_emits_nothing() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut existing = pending(organization_id);
        existing.status = ExtraStatus::Rejected;
        let id = existing.id;
        let emitter = RecordingEmitter::new();
        let mut service = service(holding(existing), MockMemberRepository::new(), &emitter);

        let approved = service.approve(organization_id, id, None).await.unwrap();
        let cancelled = service
            .cancel(organization_id, id, None, true)
            .await
            .unwrap();

        assert!(approved.is_err());
        assert!(cancelled.is_err());
        assert!(emitter.names().is_empty());
    }
}
