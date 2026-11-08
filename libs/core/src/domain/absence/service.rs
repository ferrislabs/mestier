use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};

use crate::{
    Absence, AbsenceOutcome, AbsenceSpan, AbsenceStatus, AbsenceType, AbsenceTypeService, MemberId,
    OrganizationId, Tz,
    domain::{
        absence::{
            AbsenceError, AbsenceId, clean_note, clean_proof,
            commands::{AbsenceSchedule, CreateAbsenceCommand, PatchAbsenceCommand},
            ports::AbsenceRepository,
        },
        absence_type::ports::AbsenceTypeRepository,
        leave_account::ports::LeaveAccountRepository,
    },
};

pub struct AbsenceService<AR, TR, LR>
where
    AR: AbsenceRepository,
    TR: AbsenceTypeRepository,
    LR: LeaveAccountRepository,
{
    absence_repository: AR,
    types: AbsenceTypeService<TR, LR>,
}

impl<AR, TR, LR> AbsenceService<AR, TR, LR>
where
    AR: AbsenceRepository,
    TR: AbsenceTypeRepository,
    LR: LeaveAccountRepository,
{
    pub fn new(absence_repository: AR, type_repository: TR, account_repository: LR) -> Self {
        Self {
            absence_repository,
            types: AbsenceTypeService::new(type_repository, account_repository),
        }
    }

    pub async fn create_absence(
        &mut self,
        command: CreateAbsenceCommand,
        creator: Option<MemberId>,
        timezone: Tz,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        if let AbsenceSchedule::Window {
            starts_at, ends_at, ..
        } = &command.schedule
        {
            validate_schedule(*starts_at, *ends_at)?;
            validate_note(&command.note)?;
        }

        let explicit = command.type_id.is_some();
        let absence_type = match (command.type_id, command.kind) {
            (Some(id), _) => self.types.find(command.organization_id, id).await?,
            (None, Some(kind)) => {
                match self
                    .types
                    .default_for_kind(command.organization_id, kind)
                    .await?
                {
                    Some(absence_type) => absence_type,
                    None => return Ok(Err(AbsenceError::TypeArchived)),
                }
            }
            (None, None) => return Ok(Err(AbsenceError::TypeRequired)),
        };

        let (span, window, all_day) = match command.schedule {
            AbsenceSchedule::Window {
                starts_at,
                ends_at,
                all_day,
            } => {
                let span = match AbsenceSpan::from_window(starts_at, ends_at, timezone) {
                    Ok(span) => span,
                    Err(refusal) => return Ok(Err(refusal)),
                };
                (span, (starts_at, ends_at), Some(all_day))
            }
            AbsenceSchedule::Days {
                start_date,
                start_part,
                end_date,
                end_part,
            } => {
                let span = match AbsenceSpan::new(start_date, start_part, end_date, end_part) {
                    Ok(span) => span,
                    Err(refusal) => return Ok(Err(refusal)),
                };
                let window = span.window(timezone)?;
                (span, window, None)
            }
        };

        let mut absence = match Absence::request(
            AbsenceId(generate_uuid_v7()),
            command.organization_id,
            command.member_id,
            &absence_type,
            span,
            window,
            explicit && absence_type.needs_approval,
            command.note.as_deref(),
            command.proof_file_key.as_deref(),
            creator,
            Utc::now(),
        ) {
            Ok(absence) => absence,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if let Some(all_day) = all_day {
            absence.all_day = all_day;
        }

        self.absence_repository.insert(&absence).await.map(Ok)
    }

    pub async fn get_absence(&mut self, id: AbsenceId) -> Result<Absence, CoreError> {
        self.absence_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_absences(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        self.absence_repository
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn list_absences_of_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        self.absence_repository
            .list_by_member(organization_id, member_id, limit, offset)
            .await
    }

    pub async fn patch_absence(
        &mut self,
        command: PatchAbsenceCommand,
        timezone: Tz,
    ) -> Result<Absence, CoreError> {
        let mut absence = self.get_absence(command.id).await?;

        let starts_at = command.starts_at.unwrap_or(absence.starts_at);
        let ends_at = command.ends_at.unwrap_or(absence.ends_at);
        validate_schedule(starts_at, ends_at)?;

        let note = command.note.unwrap_or_else(|| absence.note.clone());
        validate_note(&note)?;

        if let Some(proof) = command.proof_file_key {
            absence.proof_file_key =
                clean_proof(proof.as_deref()).map_err(|error| conflict(&error))?;
        }

        if command.starts_at.is_some() || command.ends_at.is_some() {
            let span = AbsenceSpan::from_window(starts_at, ends_at, timezone)
                .map_err(|error| conflict(&error))?;
            absence.start_date = span.start_date();
            absence.start_part = span.start_part();
            absence.end_date = span.end_date();
            absence.end_part = span.end_part();
        }

        if let Some(kind) = command.kind
            && kind != absence.kind
        {
            let absence_type = self
                .types
                .default_for_kind(absence.organization_id, kind)
                .await?
                .ok_or_else(|| conflict(&AbsenceError::TypeArchived))?;
            absence.type_id = absence_type.id;
            absence.kind = absence_type.kind;
        }

        absence.starts_at = starts_at;
        absence.ends_at = ends_at;
        if let Some(all_day) = command.all_day {
            absence.all_day = all_day;
        }
        absence.note = note;
        absence.updated_at = Utc::now();

        self.absence_repository.update(&absence).await
    }

    pub async fn soft_delete_absence(&mut self, id: AbsenceId) -> Result<(), CoreError> {
        self.get_absence(id).await?;
        self.absence_repository.soft_delete(id, Utc::now()).await
    }

    pub async fn approve(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceId,
        decider: Option<MemberId>,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        let mut absence = self.in_organization(organization_id, id).await?;
        let absence_type = self.type_of(&absence).await?;
        if let Err(refusal) = absence.approve(&absence_type, decider, Utc::now()) {
            return Ok(Err(refusal));
        }
        self.absence_repository.update(&absence).await.map(Ok)
    }

    pub async fn reject(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceId,
        comment: &str,
        decider: Option<MemberId>,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        let mut absence = self.in_organization(organization_id, id).await?;
        if let Err(refusal) = absence.reject(comment, decider, Utc::now()) {
            return Ok(Err(refusal));
        }
        self.absence_repository.update(&absence).await.map(Ok)
    }

    pub async fn cancel(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceId,
        caller: Option<MemberId>,
        privileged: bool,
    ) -> Result<AbsenceOutcome<Absence>, CoreError> {
        let mut absence = self.in_organization(organization_id, id).await?;
        let is_creator = caller.is_some() && absence.created_by_member_id == caller;
        if !privileged && !is_creator {
            return Err(CoreError::Forbidden {
                reason: Some("only the creator or a planning manager can cancel".to_owned()),
            });
        }
        if !privileged && absence.status != AbsenceStatus::Pending {
            return Ok(Err(AbsenceError::AlreadyDecided));
        }
        if let Err(refusal) = absence.cancel(caller, Utc::now()) {
            return Ok(Err(refusal));
        }
        self.absence_repository.update(&absence).await.map(Ok)
    }

    pub async fn in_organization(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceId,
    ) -> Result<Absence, CoreError> {
        let absence = self.get_absence(id).await?;
        if absence.organization_id == organization_id {
            Ok(absence)
        } else {
            Err(CoreError::NotFound)
        }
    }

    async fn type_of(&mut self, absence: &Absence) -> Result<AbsenceType, CoreError> {
        self.types
            .find(absence.organization_id, absence.type_id)
            .await
    }
}

fn conflict(error: &AbsenceError) -> CoreError {
    CoreError::Conflict(error.to_string())
}

fn validate_schedule(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Result<(), CoreError> {
    if ends_at <= starts_at {
        return Err(CoreError::Conflict(
            "absence ends_at must be after starts_at".to_owned(),
        ));
    }

    Ok(())
}

fn validate_note(note: &Option<String>) -> Result<(), CoreError> {
    clean_note(note.as_deref())
        .map(|_| ())
        .map_err(|_| CoreError::Conflict("absence note cannot be blank".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AbsenceKind, AbsenceStatus, AbsenceTypeId, DayPart, MemberId, OrganizationId,
        domain::{
            absence::ports::MockAbsenceRepository,
            absence_type::{
                AbsenceTypeInput, default_absence_types, ports::MockAbsenceTypeRepository,
            },
            leave_account::ports::MockLeaveAccountRepository,
        },
    };
    use chrono_tz::Europe::Paris;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn defaults(organization_id: OrganizationId) -> Vec<AbsenceType> {
        default_absence_types()
            .iter()
            .map(|default| {
                AbsenceType::new(
                    AbsenceTypeId(Uuid::new_v4()),
                    organization_id,
                    AbsenceTypeInput::from(default),
                    Utc::now(),
                )
                .unwrap()
            })
            .collect()
    }

    fn type_repository(stored: Vec<AbsenceType>) -> MockAbsenceTypeRepository {
        let mut types = MockAbsenceTypeRepository::new();
        types
            .expect_lock_organization()
            .returning(|_| Box::pin(async { Ok(()) }));
        let listed = stored.clone();
        types.expect_list_by_organization().returning(move |_| {
            let all = listed.clone();
            Box::pin(async move { Ok(all) })
        });
        types.expect_find().returning(move |_, id| {
            let found = stored.iter().find(|t| t.id == id).cloned();
            Box::pin(async move { Ok(found) })
        });
        types
    }

    fn service(
        absence_repository: MockAbsenceRepository,
        stored: Vec<AbsenceType>,
    ) -> AbsenceService<MockAbsenceRepository, MockAbsenceTypeRepository, MockLeaveAccountRepository>
    {
        AbsenceService::new(
            absence_repository,
            type_repository(stored),
            MockLeaveAccountRepository::new(),
        )
    }

    fn echo_insert(repository: &mut MockAbsenceRepository) {
        repository.expect_insert().times(1).returning(|a| {
            let cloned = a.clone();
            Box::pin(async move { Ok(cloned) })
        });
    }

    fn window_command(organization_id: OrganizationId) -> CreateAbsenceCommand {
        let now = Utc::now();
        CreateAbsenceCommand {
            actor: authz::Subject::system(),
            organization_id,
            member_id: MemberId(Uuid::new_v4()),
            kind: Some(AbsenceKind::Leave),
            type_id: None,
            schedule: AbsenceSchedule::Window {
                starts_at: now,
                ends_at: now + chrono::Duration::hours(2),
                all_day: false,
            },
            note: Some("Congés payés".to_owned()),
            proof_file_key: None,
        }
    }

    fn absence(id: AbsenceId, organization_id: OrganizationId, type_id: AbsenceTypeId) -> Absence {
        let now = Utc::now();
        let date = now.date_naive();
        Absence {
            id,
            organization_id,
            member_id: MemberId(Uuid::new_v4()),
            kind: AbsenceKind::Leave,
            type_id,
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            start_date: date,
            start_part: DayPart::Morning,
            end_date: date,
            end_part: DayPart::Afternoon,
            status: AbsenceStatus::Approved,
            note: Some("Congés payés".to_owned()),
            proof_file_key: None,
            created_by_member_id: None,
            decided_by_member_id: None,
            decided_at: None,
            decision_comment: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn repository_holding(existing: Absence) -> MockAbsenceRepository {
        let id = existing.id;
        let mut repository = MockAbsenceRepository::new();
        repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        repository
    }

    #[tokio::test]
    async fn a_legacy_request_takes_the_default_type_of_its_kind_and_is_approved() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let stored = defaults(organization_id);
        let leave_default = stored[0].id;
        let mut repository = MockAbsenceRepository::new();
        echo_insert(&mut repository);
        let mut service = service(repository, stored);

        let created = service
            .create_absence(window_command(organization_id), None, Paris)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(created.kind, AbsenceKind::Leave);
        assert_eq!(created.type_id, leave_default);
        assert_eq!(created.status, AbsenceStatus::Approved);
        assert_eq!(created.note.as_deref(), Some("Congés payés"));
    }

    #[tokio::test]
    async fn spec_rh_3_2_an_explicit_type_needing_approval_creates_pending() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let stored = defaults(organization_id);
        let sick = stored[2].clone();
        let leave = stored[0].clone();
        let mut repository = MockAbsenceRepository::new();
        echo_insert(&mut repository);
        echo_insert(&mut repository);
        let mut service = service(repository, stored);
        let days = |type_id| CreateAbsenceCommand {
            kind: None,
            type_id: Some(type_id),
            schedule: AbsenceSchedule::Days {
                start_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 6).unwrap(),
                start_part: DayPart::Morning,
                end_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
                end_part: DayPart::Afternoon,
            },
            ..window_command(organization_id)
        };

        let pending = service
            .create_absence(days(leave.id), None, Paris)
            .await
            .unwrap()
            .unwrap();
        let approved = service
            .create_absence(days(sick.id), None, Paris)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(pending.status, AbsenceStatus::Pending);
        assert!(pending.all_day);
        assert_eq!(approved.status, AbsenceStatus::Approved);
        assert_eq!(approved.kind, AbsenceKind::Sick);
    }

    #[tokio::test]
    async fn create_absence_without_type_or_kind_is_refused() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut service = service(MockAbsenceRepository::new(), defaults(organization_id));
        let mut command = window_command(organization_id);
        command.kind = None;

        let outcome = service.create_absence(command, None, Paris).await.unwrap();

        assert_eq!(outcome, Err(AbsenceError::TypeRequired));
    }

    #[tokio::test]
    async fn create_absence_rejects_ends_at_before_starts_at() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut service = service(MockAbsenceRepository::new(), defaults(organization_id));
        let mut command = window_command(organization_id);
        if let AbsenceSchedule::Window {
            starts_at, ends_at, ..
        } = &mut command.schedule
        {
            *ends_at = *starts_at;
        }

        let err = service
            .create_absence(command, None, Paris)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_absence_rejects_blank_note() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut service = service(MockAbsenceRepository::new(), defaults(organization_id));
        let mut command = window_command(organization_id);
        command.note = Some("   ".to_owned());

        let err = service
            .create_absence(command, None, Paris)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn spec_rh_3_6_an_archived_type_cannot_be_chosen() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut stored = defaults(organization_id);
        stored[0].archive(Utc::now());
        let archived = stored[0].id;
        let mut service = service(MockAbsenceRepository::new(), stored);
        let mut command = window_command(organization_id);
        command.kind = None;
        command.type_id = Some(archived);

        let outcome = service.create_absence(command, None, Paris).await.unwrap();

        assert_eq!(outcome, Err(AbsenceError::TypeArchived));
    }

    #[tokio::test]
    async fn get_absence_returns_not_found_when_missing() {
        let id = AbsenceId(Uuid::new_v4());
        let mut repository = MockAbsenceRepository::new();
        repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut service = service(repository, Vec::new());

        let err = service.get_absence(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_absences_delegates_to_repo() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repository = MockAbsenceRepository::new();
        repository
            .expect_list_by_organization()
            .with(eq(organization_id), eq(10), eq(20))
            .returning(move |org_id, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![absence(
                            AbsenceId(Uuid::new_v4()),
                            org_id,
                            AbsenceTypeId(Uuid::new_v4()),
                        )],
                        1,
                    ))
                })
            });
        let mut service = service(repository, Vec::new());

        let (items, total) = service
            .list_absences(organization_id, 10, 20)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn patch_absence_reschedules_and_changes_kind_to_the_default_type() {
        let id = AbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let stored = defaults(organization_id);
        let sick_default = stored[2].id;
        let existing = absence(id, organization_id, stored[0].id);
        let new_starts_at = existing.starts_at + chrono::Duration::days(1);
        let new_ends_at = existing.ends_at + chrono::Duration::days(1);
        let mut repository = repository_holding(existing);
        repository
            .expect_update()
            .withf(move |a| a.kind == AbsenceKind::Sick && a.type_id == sick_default)
            .returning(|a| {
                let cloned = a.clone();
                Box::pin(async move { Ok(cloned) })
            });
        let mut service = service(repository, stored);
        let mut command = PatchAbsenceCommand::new(id, authz::Subject::system());
        command.starts_at = Some(new_starts_at);
        command.ends_at = Some(new_ends_at);
        command.kind = Some(AbsenceKind::Sick);

        let updated = service.patch_absence(command, Paris).await.unwrap();

        assert_eq!(updated.starts_at, new_starts_at);
        assert_eq!(updated.ends_at, new_ends_at);
        assert_eq!(updated.kind, AbsenceKind::Sick);
    }

    #[tokio::test]
    async fn patch_absence_rejects_ends_at_before_merged_starts_at() {
        let id = AbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id, AbsenceTypeId(Uuid::new_v4()));
        let existing_starts_at = existing.starts_at;
        let mut service = service(repository_holding(existing), Vec::new());
        let mut command = PatchAbsenceCommand::new(id, authz::Subject::system());
        command.ends_at = Some(existing_starts_at - chrono::Duration::hours(1));

        let err = service.patch_absence(command, Paris).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_absence_can_clear_the_note() {
        let id = AbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id, AbsenceTypeId(Uuid::new_v4()));
        let mut repository = repository_holding(existing);
        repository
            .expect_update()
            .withf(|a| a.note.is_none())
            .returning(|a| {
                let cloned = a.clone();
                Box::pin(async move { Ok(cloned) })
            });
        let mut service = service(repository, Vec::new());
        let mut command = PatchAbsenceCommand::new(id, authz::Subject::system());
        command.note = Some(None);

        let updated = service.patch_absence(command, Paris).await.unwrap();

        assert!(updated.note.is_none());
    }

    #[tokio::test]
    async fn patch_absence_rejects_blank_note() {
        let id = AbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id, AbsenceTypeId(Uuid::new_v4()));
        let mut service = service(repository_holding(existing), Vec::new());
        let mut command = PatchAbsenceCommand::new(id, authz::Subject::system());
        command.note = Some(Some("   ".to_owned()));

        let err = service.patch_absence(command, Paris).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn soft_delete_absence_checks_existence_then_deletes() {
        let id = AbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id, AbsenceTypeId(Uuid::new_v4()));
        let mut repository = repository_holding(existing);
        repository
            .expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let mut service = service(repository, Vec::new());

        service.soft_delete_absence(id).await.unwrap();
    }

    #[tokio::test]
    async fn soft_delete_absence_returns_not_found_when_missing() {
        let id = AbsenceId(Uuid::new_v4());
        let mut repository = MockAbsenceRepository::new();
        repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut service = service(repository, Vec::new());

        let err = service.soft_delete_absence(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn an_absence_of_another_organization_answers_not_found_on_decisions() {
        let id = AbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id, AbsenceTypeId(Uuid::new_v4()));
        let mut service = service(repository_holding(existing), Vec::new());

        let err = service
            .approve(OrganizationId(Uuid::new_v4()), id, None)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn only_the_creator_or_a_privileged_caller_can_cancel() {
        let id = AbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let creator = MemberId(Uuid::new_v4());
        let mut existing = absence(id, organization_id, AbsenceTypeId(Uuid::new_v4()));
        existing.created_by_member_id = Some(creator);
        existing.status = AbsenceStatus::Pending;
        let mut repository = repository_holding(existing);
        repository.expect_update().times(2).returning(|a| {
            let cloned = a.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let mut service = service(repository, Vec::new());

        let stranger = service
            .cancel(organization_id, id, Some(MemberId(Uuid::new_v4())), false)
            .await
            .unwrap_err();
        let by_creator = service
            .cancel(organization_id, id, Some(creator), false)
            .await
            .unwrap()
            .unwrap();
        let by_manager = service
            .cancel(organization_id, id, None, true)
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(stranger, CoreError::Forbidden { .. }));
        assert_eq!(by_creator.status, AbsenceStatus::Cancelled);
        assert_eq!(by_manager.status, AbsenceStatus::Cancelled);
    }
}
