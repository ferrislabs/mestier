use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, MemberId, OrganizationId,
    domain::assignment_report::{
        commands::{
            AmendAssignmentReportCommand, ReportAssignmentCommand, ResolveAssignmentReportCommand,
            WithdrawAssignmentReportCommand,
        },
        events::{AssignmentReportApplied, AssignmentReportDismissed, AssignmentReportReported},
        ports::AssignmentReportRepository,
    },
};

/// Mirrors `chk_assignment_reports_comment_not_blank_when_present` /
/// `..._resolution_note_not_blank_when_present`, ahead of the trip to the
/// database — same shape as `task_comment::service::validate_body`.
fn validate_optional_text(value: &Option<String>, what: &str) -> Result<(), CoreError> {
    if let Some(text) = value
        && text.trim().is_empty()
    {
        return Err(CoreError::Conflict(format!("{what} cannot be blank")));
    }
    Ok(())
}

pub struct AssignmentReportService<R, E>
where
    R: AssignmentReportRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> AssignmentReportService<R, E>
where
    R: AssignmentReportRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    /// Files a report against the caller's own assignment.
    ///
    /// Refuses when the assignment does not exist, or when it belongs to a
    /// member other than the caller — the security rule this workstream
    /// exists to enforce. `reported_minutes` is `u32`, so "cannot be
    /// negative" is a type-level fact rather than a runtime check; zero is a
    /// legitimate answer ("this did not happen").
    pub async fn report_assignment(
        &mut self,
        command: ReportAssignmentCommand,
    ) -> Result<AssignmentReport, CoreError> {
        validate_optional_text(&command.comment, "assignment report comment")?;

        let context = self
            .repo
            .find_assignment_context(command.task_assignment_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if context.member_id != command.reported_by {
            return Err(CoreError::Forbidden {
                reason: Some("only the assignee may report on their own assignment".to_owned()),
            });
        }

        let now = Utc::now();
        let report = self
            .repo
            .insert(&AssignmentReport {
                id: AssignmentReportId(generate_uuid_v7()),
                organization_id: context.organization_id,
                task_assignment_id: command.task_assignment_id,
                reported_minutes: command.reported_minutes,
                comment: command.comment,
                reported_by: command.reported_by,
                resolution: AssignmentReportResolution::Pending,
                resolved_by: None,
                resolved_at: None,
                resolution_note: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.emitter.emit(
            report.organization_id,
            &AssignmentReportReported {
                report: report.clone(),
            },
        )?;

        Ok(report)
    }

    pub async fn get_report(
        &mut self,
        id: AssignmentReportId,
    ) -> Result<AssignmentReport, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    /// The reporter's own history — resolved reports included, so a worker
    /// can see that their word was acted on. See the API issue's "that last
    /// part is what makes people keep reporting."
    pub async fn list_reports_by_reporter(
        &mut self,
        organization_id: OrganizationId,
        reported_by: MemberId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<AssignmentReport>, u64), CoreError> {
        self.repo
            .list_by_reporter(organization_id, reported_by, resolution, limit, offset)
            .await
    }

    pub async fn list_reports_by_organization(
        &mut self,
        organization_id: OrganizationId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<AssignmentReport>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, resolution, limit, offset)
            .await
    }

    /// Amends a still-pending report — only its own author may, and only
    /// while nobody has acted on it yet. Emits nothing: an amendment is not
    /// a business act anyone downstream needs to react to.
    pub async fn amend_report(
        &mut self,
        command: AmendAssignmentReportCommand,
    ) -> Result<AssignmentReport, CoreError> {
        validate_optional_text(&command.comment, "assignment report comment")?;

        let mut report = self.get_report(command.id).await?;
        if report.reported_by != command.acting_member_id {
            return Err(CoreError::Forbidden {
                reason: Some("only the report's author may amend it".to_owned()),
            });
        }
        if !report.is_pending() {
            return Err(CoreError::Conflict(
                "a resolved report can no longer be amended".to_owned(),
            ));
        }

        report.reported_minutes = command.reported_minutes;
        report.comment = command.comment;
        report.updated_at = Utc::now();

        self.repo.update(&report).await
    }

    /// Withdraws a still-pending report — same authorship and pending rules
    /// as [`Self::amend_report`]. Physical delete, so nothing downstream
    /// sees the report again.
    pub async fn withdraw_report(
        &mut self,
        command: WithdrawAssignmentReportCommand,
    ) -> Result<(), CoreError> {
        let report = self.get_report(command.id).await?;
        if report.reported_by != command.acting_member_id {
            return Err(CoreError::Forbidden {
                reason: Some("only the report's author may withdraw it".to_owned()),
            });
        }
        if !report.is_pending() {
            return Err(CoreError::Conflict(
                "a resolved report can no longer be withdrawn".to_owned(),
            ));
        }

        self.repo.delete(command.id).await
    }

    /// The manager's arbitration.
    ///
    /// Refuses a report that is not pending, with its own error rather than
    /// a silent no-op — resolving twice must never look like it succeeded
    /// twice. Refuses `Pending` as a target resolution: it is not a decision
    /// a manager makes *into*.
    ///
    /// Does not touch the task: recording the decision and moving the plan
    /// are two separate acts, which is what makes "applied" auditable — see
    /// the issue's "Model" section.
    pub async fn resolve_report(
        &mut self,
        command: ResolveAssignmentReportCommand,
    ) -> Result<AssignmentReport, CoreError> {
        if command.resolution == AssignmentReportResolution::Pending {
            return Err(CoreError::Conflict(
                "a report cannot be resolved into `PENDING`".to_owned(),
            ));
        }
        validate_optional_text(&command.resolution_note, "resolution note")?;

        let mut report = self.get_report(command.id).await?;
        if !report.is_pending() {
            return Err(CoreError::Conflict(
                "this report has already been resolved".to_owned(),
            ));
        }

        let resolved_at = Utc::now();
        report.resolution = command.resolution;
        report.resolved_by = Some(command.resolved_by);
        report.resolved_at = Some(resolved_at);
        report.resolution_note = command.resolution_note;
        report.updated_at = resolved_at;

        let resolved = self.repo.update(&report).await?;

        match resolved.resolution {
            AssignmentReportResolution::Applied => {
                self.emitter.emit(
                    resolved.organization_id,
                    &AssignmentReportApplied {
                        report: resolved.clone(),
                    },
                )?;
            }
            AssignmentReportResolution::Dismissed => {
                self.emitter.emit(
                    resolved.organization_id,
                    &AssignmentReportDismissed {
                        report: resolved.clone(),
                    },
                )?;
            }
            AssignmentReportResolution::Pending => {
                unreachable!("guarded above: command.resolution is never Pending here")
            }
        }

        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use events::testing::RecordingEmitter;
    use mockall::predicate::eq;
    use uuid::Uuid;

    use super::*;
    use crate::{
        TaskAssignmentId, TaskId,
        domain::assignment_report::ports::{AssignmentContext, MockAssignmentReportRepository},
    };

    fn member() -> MemberId {
        MemberId(Uuid::new_v4())
    }

    fn assignment_id() -> TaskAssignmentId {
        TaskAssignmentId(Uuid::new_v4())
    }

    fn context(member_id: MemberId) -> AssignmentContext {
        AssignmentContext {
            organization_id: OrganizationId(Uuid::new_v4()),
            task_id: TaskId(Uuid::new_v4()),
            member_id,
        }
    }

    fn report(
        id: AssignmentReportId,
        task_assignment_id: TaskAssignmentId,
        reported_by: MemberId,
        resolution: AssignmentReportResolution,
    ) -> AssignmentReport {
        let now = Utc::now();
        AssignmentReport {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            task_assignment_id,
            reported_minutes: 180,
            comment: None,
            reported_by,
            resolution,
            resolved_by: None,
            resolved_at: None,
            resolution_note: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn report_command(
        task_assignment_id: TaskAssignmentId,
        reported_by: MemberId,
    ) -> ReportAssignmentCommand {
        ReportAssignmentCommand {
            task_assignment_id,
            reported_by,
            reported_minutes: 300,
            comment: Some("Chantier plus long que prévu".to_owned()),
        }
    }

    // -- report_assignment ----------------------------------------------------

    #[tokio::test]
    async fn report_assignment_persists_and_emits_when_the_caller_is_the_assignee() {
        let member_id = member();
        let task_assignment_id = assignment_id();
        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_assignment_context()
            .with(eq(task_assignment_id))
            .returning(move |_| {
                let ctx = context(member_id);
                Box::pin(async move { Ok(Some(ctx)) })
            });
        repo.expect_insert().times(1).returning(|r| {
            let cloned = r.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = AssignmentReportService::new(repo, &emitter);

        let created = service
            .report_assignment(report_command(task_assignment_id, member_id))
            .await
            .unwrap();

        assert_eq!(created.reported_minutes, 300);
        assert_eq!(created.reported_by, member_id);
        assert!(created.is_pending());
        assert_eq!(emitter.names(), vec!["assignment_report.reported"]);
    }

    #[tokio::test]
    async fn report_assignment_rejects_a_blank_comment() {
        let member_id = member();
        let mut service = AssignmentReportService::new(
            MockAssignmentReportRepository::new(),
            RecordingEmitter::new(),
        );

        let mut command = report_command(assignment_id(), member_id);
        command.comment = Some("   ".to_owned());

        let err = service.report_assignment(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn report_assignment_returns_not_found_when_the_assignment_does_not_exist() {
        let task_assignment_id = assignment_id();
        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_assignment_context()
            .with(eq(task_assignment_id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        let err = service
            .report_assignment(report_command(task_assignment_id, member()))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    /// The security rule this workstream exists to enforce: reporting on
    /// someone else's assignment must be refused, not silently allowed.
    #[tokio::test]
    async fn report_assignment_forbids_reporting_on_someone_elses_assignment() {
        let assignee_member_id = member();
        let caller_member_id = member();
        let task_assignment_id = assignment_id();
        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_assignment_context()
            .with(eq(task_assignment_id))
            .returning(move |_| {
                let ctx = context(assignee_member_id);
                Box::pin(async move { Ok(Some(ctx)) })
            });
        // No `expect_insert`: a mismatched caller must be rejected before
        // any write is attempted.

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        let err = service
            .report_assignment(report_command(task_assignment_id, caller_member_id))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    // -- amend_report -----------------------------------------------------------

    #[tokio::test]
    async fn amend_report_mutates_when_pending_and_the_author_matches() {
        let member_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            member_id,
            AssignmentReportResolution::Pending,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });
        repo.expect_update()
            .withf(|r| r.reported_minutes == 240)
            .returning(|r| {
                let cloned = r.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        let amended = service
            .amend_report(AmendAssignmentReportCommand {
                id,
                acting_member_id: member_id,
                reported_minutes: 240,
                comment: None,
            })
            .await
            .unwrap();

        assert_eq!(amended.reported_minutes, 240);
    }

    #[tokio::test]
    async fn amend_report_forbids_amending_someone_elses_report() {
        let author_id = member();
        let other_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            author_id,
            AssignmentReportResolution::Pending,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        let err = service
            .amend_report(AmendAssignmentReportCommand {
                id,
                acting_member_id: other_id,
                reported_minutes: 100,
                comment: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn amend_report_refuses_a_report_that_is_already_resolved() {
        let member_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            member_id,
            AssignmentReportResolution::Applied,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        let err = service
            .amend_report(AmendAssignmentReportCommand {
                id,
                acting_member_id: member_id,
                reported_minutes: 100,
                comment: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    // -- withdraw_report ----------------------------------------------------

    #[tokio::test]
    async fn withdraw_report_deletes_when_pending_and_the_author_matches() {
        let member_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            member_id,
            AssignmentReportResolution::Pending,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });
        repo.expect_delete()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        service
            .withdraw_report(WithdrawAssignmentReportCommand {
                id,
                acting_member_id: member_id,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn withdraw_report_refuses_a_report_that_is_already_resolved() {
        let member_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            member_id,
            AssignmentReportResolution::Dismissed,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });
        // No `expect_delete`: a resolved report must never be removed by a
        // withdraw call.

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        let err = service
            .withdraw_report(WithdrawAssignmentReportCommand {
                id,
                acting_member_id: member_id,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    // -- resolve_report -----------------------------------------------------

    #[tokio::test]
    async fn resolve_report_applies_and_emits_when_pending() {
        let member_id = member();
        let manager_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            member_id,
            AssignmentReportResolution::Pending,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });
        repo.expect_update()
            .withf(|r| r.resolution == AssignmentReportResolution::Applied)
            .returning(|r| {
                let cloned = r.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let emitter = RecordingEmitter::new();
        let mut service = AssignmentReportService::new(repo, &emitter);
        let resolved = service
            .resolve_report(ResolveAssignmentReportCommand {
                id,
                resolved_by: manager_id,
                resolution: AssignmentReportResolution::Applied,
                resolution_note: None,
            })
            .await
            .unwrap();

        assert_eq!(resolved.resolution, AssignmentReportResolution::Applied);
        assert_eq!(resolved.resolved_by, Some(manager_id));
        assert!(resolved.resolved_at.is_some());
        assert_eq!(emitter.names(), vec!["assignment_report.applied"]);
    }

    #[tokio::test]
    async fn resolve_report_dismisses_and_emits_when_pending() {
        let member_id = member();
        let manager_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            member_id,
            AssignmentReportResolution::Pending,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });
        repo.expect_update().returning(|r| {
            let cloned = r.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let emitter = RecordingEmitter::new();
        let mut service = AssignmentReportService::new(repo, &emitter);
        let resolved = service
            .resolve_report(ResolveAssignmentReportCommand {
                id,
                resolved_by: manager_id,
                resolution: AssignmentReportResolution::Dismissed,
                resolution_note: Some("Écart déjà couvert par un avenant".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(resolved.resolution, AssignmentReportResolution::Dismissed);
        assert_eq!(emitter.names(), vec!["assignment_report.dismissed"]);
    }

    #[tokio::test]
    async fn resolve_report_refuses_resolving_into_pending() {
        let mut service = AssignmentReportService::new(
            MockAssignmentReportRepository::new(),
            RecordingEmitter::new(),
        );

        let err = service
            .resolve_report(ResolveAssignmentReportCommand {
                id: AssignmentReportId(Uuid::new_v4()),
                resolved_by: member(),
                resolution: AssignmentReportResolution::Pending,
                resolution_note: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    /// Resolving twice must fail loudly rather than silently no-op.
    #[tokio::test]
    async fn resolve_report_refuses_a_report_that_is_already_resolved() {
        let member_id = member();
        let id = AssignmentReportId(Uuid::new_v4());
        let existing = report(
            id,
            assignment_id(),
            member_id,
            AssignmentReportResolution::Applied,
        );

        let mut repo = MockAssignmentReportRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let r = existing.clone();
            Box::pin(async move { Ok(Some(r)) })
        });
        // No `expect_update`: resolving an already-resolved report must not
        // write anything.

        let mut service = AssignmentReportService::new(repo, RecordingEmitter::new());
        let err = service
            .resolve_report(ResolveAssignmentReportCommand {
                id,
                resolved_by: member(),
                resolution: AssignmentReportResolution::Applied,
                resolution_note: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }
}
