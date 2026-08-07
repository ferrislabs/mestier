use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    EmployeeAbsence,
    domain::absence::{
        EmployeeAbsenceId,
        commands::{CreateAbsenceCommand, PatchAbsenceCommand},
        ports::AbsenceRepository,
    },
};

pub struct AbsenceService<AR>
where
    AR: AbsenceRepository,
{
    absence_repository: AR,
}

impl<AR> AbsenceService<AR>
where
    AR: AbsenceRepository,
{
    pub fn new(absence_repository: AR) -> Self {
        Self { absence_repository }
    }

    pub async fn create_absence(
        &mut self,
        command: CreateAbsenceCommand,
    ) -> Result<EmployeeAbsence, CoreError> {
        validate_schedule(command.starts_at, command.ends_at)?;
        validate_note(&command.note)?;

        let now = Utc::now();
        self.absence_repository
            .insert(&EmployeeAbsence {
                id: EmployeeAbsenceId(generate_uuid_v7()),
                organization_id: command.organization_id,
                employee_id: command.employee_id,
                kind: command.kind,
                starts_at: command.starts_at,
                ends_at: command.ends_at,
                all_day: command.all_day,
                note: command.note,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_absence(
        &mut self,
        id: EmployeeAbsenceId,
    ) -> Result<EmployeeAbsence, CoreError> {
        self.absence_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_absences(
        &mut self,
        organization_id: crate::OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<EmployeeAbsence>, u64), CoreError> {
        self.absence_repository
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn patch_absence(
        &mut self,
        command: PatchAbsenceCommand,
    ) -> Result<EmployeeAbsence, CoreError> {
        let mut absence = self.get_absence(command.id).await?;

        let starts_at = command.starts_at.unwrap_or(absence.starts_at);
        let ends_at = command.ends_at.unwrap_or(absence.ends_at);
        validate_schedule(starts_at, ends_at)?;

        let note = command.note.unwrap_or_else(|| absence.note.clone());
        validate_note(&note)?;

        absence.starts_at = starts_at;
        absence.ends_at = ends_at;
        if let Some(all_day) = command.all_day {
            absence.all_day = all_day;
        }
        if let Some(kind) = command.kind {
            absence.kind = kind;
        }
        absence.note = note;
        absence.updated_at = Utc::now();

        self.absence_repository.update(&absence).await
    }

    pub async fn soft_delete_absence(&mut self, id: EmployeeAbsenceId) -> Result<(), CoreError> {
        self.get_absence(id).await?;
        self.absence_repository.soft_delete(id, Utc::now()).await
    }
}

fn validate_schedule(
    starts_at: chrono::DateTime<Utc>,
    ends_at: chrono::DateTime<Utc>,
) -> Result<(), CoreError> {
    if ends_at <= starts_at {
        return Err(CoreError::Conflict(
            "absence ends_at must be after starts_at".to_owned(),
        ));
    }

    Ok(())
}

fn validate_note(note: &Option<String>) -> Result<(), CoreError> {
    if note.as_deref().is_some_and(|v| v.trim().is_empty()) {
        return Err(CoreError::Conflict(
            "absence note cannot be blank".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EmployeeId, OrganizationId, domain::absence::ports::MockAbsenceRepository};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn service(absence_repository: MockAbsenceRepository) -> AbsenceService<MockAbsenceRepository> {
        AbsenceService::new(absence_repository)
    }

    fn create_command() -> CreateAbsenceCommand {
        let now = Utc::now();
        CreateAbsenceCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id: EmployeeId(Uuid::new_v4()),
            kind: crate::AbsenceKind::Leave,
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            note: Some("Congés payés".to_owned()),
        }
    }

    fn absence(id: EmployeeAbsenceId, organization_id: OrganizationId) -> EmployeeAbsence {
        let now = Utc::now();
        EmployeeAbsence {
            id,
            organization_id,
            employee_id: EmployeeId(Uuid::new_v4()),
            kind: crate::AbsenceKind::Leave,
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            note: Some("Congés payés".to_owned()),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_absence_persists_with_given_kind() {
        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository.expect_insert().times(1).returning(|a| {
            let cloned = a.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = service(absence_repository);

        let created = service.create_absence(create_command()).await.unwrap();

        assert_eq!(created.kind, crate::AbsenceKind::Leave);
        assert_eq!(created.note.as_deref(), Some("Congés payés"));
    }

    #[tokio::test]
    async fn create_absence_rejects_ends_at_before_starts_at() {
        let mut service = service(MockAbsenceRepository::new());

        let mut command = create_command();
        command.ends_at = command.starts_at;

        let err = service.create_absence(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_absence_rejects_blank_note() {
        let mut service = service(MockAbsenceRepository::new());

        let mut command = create_command();
        command.note = Some("   ".to_owned());

        let err = service.create_absence(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_absence_returns_not_found_when_missing() {
        let id = EmployeeAbsenceId(Uuid::new_v4());
        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = service(absence_repository);

        let err = service.get_absence(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_absences_delegates_to_repo() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_list_by_organization()
            .with(eq(organization_id), eq(10), eq(20))
            .returning(move |org_id, _, _| {
                Box::pin(async move {
                    Ok((vec![absence(EmployeeAbsenceId(Uuid::new_v4()), org_id)], 1))
                })
            });

        let mut service = service(absence_repository);

        let (items, total) = service
            .list_absences(organization_id, 10, 20)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn patch_absence_reschedules_and_changes_kind() {
        let id = EmployeeAbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id);
        let new_starts_at = existing.starts_at + chrono::Duration::days(1);
        let new_ends_at = existing.ends_at + chrono::Duration::days(1);

        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        absence_repository
            .expect_update()
            .withf(|a| a.kind == crate::AbsenceKind::Sick)
            .returning(|a| {
                let cloned = a.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(absence_repository);

        let mut command = PatchAbsenceCommand::new(id);
        command.starts_at = Some(new_starts_at);
        command.ends_at = Some(new_ends_at);
        command.kind = Some(crate::AbsenceKind::Sick);

        let updated = service.patch_absence(command).await.unwrap();

        assert_eq!(updated.starts_at, new_starts_at);
        assert_eq!(updated.ends_at, new_ends_at);
        assert_eq!(updated.kind, crate::AbsenceKind::Sick);
    }

    #[tokio::test]
    async fn patch_absence_rejects_ends_at_before_merged_starts_at() {
        let id = EmployeeAbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id);
        let existing_starts_at = existing.starts_at;

        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(absence_repository);

        let mut command = PatchAbsenceCommand::new(id);
        command.ends_at = Some(existing_starts_at - chrono::Duration::hours(1));

        let err = service.patch_absence(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn patch_absence_can_clear_the_note() {
        let id = EmployeeAbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id);

        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        absence_repository
            .expect_update()
            .withf(|a| a.note.is_none())
            .returning(|a| {
                let cloned = a.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(absence_repository);

        let mut command = PatchAbsenceCommand::new(id);
        command.note = Some(None);

        let updated = service.patch_absence(command).await.unwrap();

        assert!(updated.note.is_none());
    }

    #[tokio::test]
    async fn patch_absence_rejects_blank_note() {
        let id = EmployeeAbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id);

        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(absence_repository);

        let mut command = PatchAbsenceCommand::new(id);
        command.note = Some(Some("   ".to_owned()));

        let err = service.patch_absence(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn soft_delete_absence_checks_existence_then_deletes() {
        let id = EmployeeAbsenceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = absence(id, organization_id);

        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        absence_repository
            .expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = service(absence_repository);

        service.soft_delete_absence(id).await.unwrap();
    }

    #[tokio::test]
    async fn soft_delete_absence_returns_not_found_when_missing() {
        let id = EmployeeAbsenceId(Uuid::new_v4());
        let mut absence_repository = MockAbsenceRepository::new();
        absence_repository
            .expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = service(absence_repository);

        let err = service.soft_delete_absence(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }
}
