use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    MemberId, OrganizationId, SchoolTutor, SchoolTutorId, SchoolTutorView,
    domain::cursus_school_tutor::{
        NOT_A_SCHOOL_TUTOR,
        commands::{CreateSchoolTutorCommand, SchoolTutorFilter, UpdateSchoolTutorCommand},
        events::{SchoolTutorArchived, SchoolTutorDeclared},
        ports::CursusSchoolTutorRepository,
    },
};

const DUPLICATE_MEMBER_CONSTRAINT: &str = "uq_cursus_school_tutor_member";
const ALREADY_DECLARED: &str = "this member is already a school tutor";

pub struct CursusSchoolTutorService<R, E>
where
    R: CursusSchoolTutorRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> CursusSchoolTutorService<R, E>
where
    R: CursusSchoolTutorRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    pub async fn create(
        &mut self,
        command: CreateSchoolTutorCommand,
    ) -> Result<SchoolTutorView, CoreError> {
        let organization_id = command.organization_id;
        if self
            .repo
            .find_by_member(organization_id, command.member_id)
            .await?
            .is_some()
        {
            return Err(CoreError::Conflict(ALREADY_DECLARED.to_owned()));
        }

        let now = Utc::now();
        let inserted = self
            .repo
            .insert(&SchoolTutor {
                id: SchoolTutorId(generate_uuid_v7()),
                organization_id,
                member_id: command.member_id,
                job_title: optional(command.job_title),
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
            .map_err(|error| match error {
                CoreError::Conflict(constraint) if constraint == DUPLICATE_MEMBER_CONSTRAINT => {
                    CoreError::Conflict(ALREADY_DECLARED.to_owned())
                }
                other => other,
            })?;

        self.emitter
            .emit(organization_id, &SchoolTutorDeclared { tutor: &inserted })?;

        self.get(organization_id, inserted.id).await
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> Result<SchoolTutorView, CoreError> {
        self.repo
            .find(organization_id, id)
            .await?
            .filter(|found| found.tutor.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &SchoolTutorFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SchoolTutorView>, u64), CoreError> {
        self.repo.list(organization_id, filter, limit, offset).await
    }

    pub async fn update(
        &mut self,
        command: UpdateSchoolTutorCommand,
    ) -> Result<SchoolTutorView, CoreError> {
        let mut tutor = self.get(command.organization_id, command.id).await?.tutor;
        tutor.job_title = optional(command.job_title);
        tutor.updated_at = Utc::now();
        self.repo.update(&tutor).await?;

        self.get(command.organization_id, command.id).await
    }

    pub async fn archive(
        &mut self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> Result<SchoolTutorView, CoreError> {
        let current = self.get(organization_id, id).await?;
        if !current.tutor.is_active() {
            return Ok(current);
        }

        let now = Utc::now();
        let archived = self
            .repo
            .set_archived(organization_id, id, Some(now), now)
            .await?;
        self.emitter
            .emit(organization_id, &SchoolTutorArchived { tutor: &archived })?;

        self.get(organization_id, id).await
    }

    pub async fn unarchive(
        &mut self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> Result<SchoolTutorView, CoreError> {
        let current = self.get(organization_id, id).await?;
        if current.tutor.is_active() {
            return Ok(current);
        }

        self.repo
            .set_archived(organization_id, id, None, Utc::now())
            .await?;

        self.get(organization_id, id).await
    }

    pub async fn ensure_active(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<(), CoreError> {
        match self.repo.find_by_member(organization_id, member_id).await? {
            Some(tutor) if tutor.is_active() => Ok(()),
            _ => Err(CoreError::Unprocessable {
                code: NOT_A_SCHOOL_TUTOR,
                message: "the pedagogic tutor must be an active school tutor of the organization"
                    .to_owned(),
            }),
        }
    }
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use events::testing::RecordingEmitter;
    use mockall::predicate::eq;
    use uuid::Uuid;

    use super::*;
    use crate::domain::cursus_school_tutor::ports::MockCursusSchoolTutorRepository;

    fn org() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn actor() -> authz::Subject {
        crate::application::policy::user_subject(crate::UserId(Uuid::new_v4()), Vec::new())
    }

    fn tutor_in(organization_id: OrganizationId, id: SchoolTutorId) -> SchoolTutor {
        let now = Utc::now();
        SchoolTutor {
            id,
            organization_id,
            member_id: MemberId(Uuid::new_v4()),
            job_title: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn view_of(tutor: SchoolTutor) -> SchoolTutorView {
        SchoolTutorView {
            tutor,
            member_name: "Bonnal Baptiste".to_owned(),
            member_email: None,
            open_triad_count: 0,
        }
    }

    fn create_command(organization_id: OrganizationId) -> CreateSchoolTutorCommand {
        CreateSchoolTutorCommand {
            actor: actor(),
            organization_id,
            member_id: MemberId(Uuid::new_v4()),
            job_title: Some("  Formateur référent  ".to_owned()),
        }
    }

    #[tokio::test]
    async fn create_trims_the_job_title_and_emits_declared() {
        let organization_id = org();
        let command = create_command(organization_id);
        let member_id = command.member_id;
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find_by_member()
            .with(eq(organization_id), eq(member_id))
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_insert()
            .withf(move |tutor| {
                tutor.job_title.as_deref() == Some("Formateur référent")
                    && tutor.member_id == member_id
                    && tutor.organization_id == organization_id
                    && tutor.archived_at.is_none()
            })
            .times(1)
            .returning(|tutor| {
                let cloned = tutor.clone();
                Box::pin(async move { Ok(cloned) })
            });
        repo.expect_find().returning(|organization_id, id| {
            let view = view_of(tutor_in(organization_id, id));
            Box::pin(async move { Ok(Some(view)) })
        });
        let emitter = RecordingEmitter::new();

        let created = CursusSchoolTutorService::new(repo, &emitter)
            .create(command)
            .await
            .unwrap();

        assert_eq!(created.tutor.organization_id, organization_id);
        assert_eq!(emitter.names(), ["cursus.school_tutor.declared"]);
    }

    #[tokio::test]
    async fn create_turns_a_blank_job_title_into_none() {
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find_by_member()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_insert()
            .withf(|tutor| tutor.job_title.is_none())
            .times(1)
            .returning(|tutor| {
                let cloned = tutor.clone();
                Box::pin(async move { Ok(cloned) })
            });
        repo.expect_find().returning(|organization_id, id| {
            let view = view_of(tutor_in(organization_id, id));
            Box::pin(async move { Ok(Some(view)) })
        });
        let mut command = create_command(org());
        command.job_title = Some("   ".to_owned());

        CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .create(command)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn create_refuses_a_member_already_declared() {
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find_by_member()
            .returning(|organization_id, member_id| {
                let mut existing = tutor_in(organization_id, SchoolTutorId(Uuid::new_v4()));
                existing.member_id = member_id;
                Box::pin(async move { Ok(Some(existing)) })
            });
        repo.expect_insert().never();
        let emitter = RecordingEmitter::new();

        let result = CursusSchoolTutorService::new(repo, &emitter)
            .create(create_command(org()))
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(message)) if message == ALREADY_DECLARED));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn create_maps_the_unique_violation_to_a_readable_conflict() {
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find_by_member()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_insert().returning(|_| {
            Box::pin(async { Err(CoreError::Conflict(DUPLICATE_MEMBER_CONSTRAINT.to_owned())) })
        });

        let result = CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .create(create_command(org()))
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(message)) if message == ALREADY_DECLARED));
    }

    #[tokio::test]
    async fn get_hides_a_tutor_of_another_organization() {
        let asked = org();
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find().returning(|_, id| {
            let foreign = view_of(tutor_in(org(), id));
            Box::pin(async move { Ok(Some(foreign)) })
        });

        let result = CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .get(asked, SchoolTutorId(Uuid::new_v4()))
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn update_replaces_the_job_title() {
        let organization_id = org();
        let id = SchoolTutorId(Uuid::new_v4());
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find().returning(|organization_id, id| {
            let view = view_of(tutor_in(organization_id, id));
            Box::pin(async move { Ok(Some(view)) })
        });
        repo.expect_update()
            .withf(|tutor| tutor.job_title.as_deref() == Some("Responsable pédagogique"))
            .times(1)
            .returning(|tutor| {
                let cloned = tutor.clone();
                Box::pin(async move { Ok(cloned) })
            });

        CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .update(UpdateSchoolTutorCommand {
                actor: actor(),
                organization_id,
                id,
                job_title: Some(" Responsable pédagogique ".to_owned()),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn archive_emits_once_and_keeps_an_already_archived_tutor_quiet() {
        let organization_id = org();
        let id = SchoolTutorId(Uuid::new_v4());
        let mut repo = MockCursusSchoolTutorRepository::new();
        let mut archived_once = false;
        repo.expect_find().returning(move |organization_id, id| {
            let mut tutor = tutor_in(organization_id, id);
            if archived_once {
                tutor.archived_at = Some(Utc::now());
            }
            archived_once = true;
            let view = view_of(tutor);
            Box::pin(async move { Ok(Some(view)) })
        });
        repo.expect_set_archived()
            .withf(|_, _, archived_at, _| archived_at.is_some())
            .times(1)
            .returning(|organization_id, id, archived_at, _| {
                let mut tutor = tutor_in(organization_id, id);
                tutor.archived_at = archived_at;
                Box::pin(async move { Ok(tutor) })
            });
        let emitter = RecordingEmitter::new();
        let mut service = CursusSchoolTutorService::new(repo, &emitter);

        service.archive(organization_id, id).await.unwrap();
        service.archive(organization_id, id).await.unwrap();

        assert_eq!(emitter.names(), ["cursus.school_tutor.archived"]);
    }

    #[tokio::test]
    async fn unarchive_clears_the_archive_date() {
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find().returning(|organization_id, id| {
            let mut tutor = tutor_in(organization_id, id);
            tutor.archived_at = Some(Utc::now());
            let view = view_of(tutor);
            Box::pin(async move { Ok(Some(view)) })
        });
        repo.expect_set_archived()
            .withf(|_, _, archived_at, _| archived_at.is_none())
            .times(1)
            .returning(|organization_id, id, _, _| {
                let tutor = tutor_in(organization_id, id);
                Box::pin(async move { Ok(tutor) })
            });

        CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .unarchive(org(), SchoolTutorId(Uuid::new_v4()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn ensure_active_accepts_an_active_school_tutor() {
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find_by_member()
            .returning(|organization_id, _| {
                let tutor = tutor_in(organization_id, SchoolTutorId(Uuid::new_v4()));
                Box::pin(async move { Ok(Some(tutor)) })
            });

        let result = CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .ensure_active(org(), MemberId(Uuid::new_v4()))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn ensure_active_rejects_a_member_who_is_not_a_school_tutor() {
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find_by_member()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let result = CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .ensure_active(org(), MemberId(Uuid::new_v4()))
            .await;

        assert!(matches!(
            result,
            Err(CoreError::Unprocessable { code, .. }) if code == NOT_A_SCHOOL_TUTOR
        ));
    }

    #[tokio::test]
    async fn ensure_active_rejects_an_archived_school_tutor() {
        let mut repo = MockCursusSchoolTutorRepository::new();
        repo.expect_find_by_member()
            .returning(|organization_id, _| {
                let mut tutor = tutor_in(organization_id, SchoolTutorId(Uuid::new_v4()));
                tutor.archived_at = Some(Utc::now());
                Box::pin(async move { Ok(Some(tutor)) })
            });

        let result = CursusSchoolTutorService::new(repo, RecordingEmitter::new())
            .ensure_active(org(), MemberId(Uuid::new_v4()))
            .await;

        assert!(matches!(
            result,
            Err(CoreError::Unprocessable { code, .. }) if code == NOT_A_SCHOOL_TUTOR
        ));
    }
}
