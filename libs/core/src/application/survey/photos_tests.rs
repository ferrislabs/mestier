use std::sync::{Arc, Mutex};

use authz::Subject;
use chrono::{Duration, Utc};
use common::{CoreError, RefusalKind};
use uuid::Uuid;

use super::tests::{Fixture, not_found, seed_member, seed_user, survey_command, with_fixture};
use crate::application::MestierUseCase;
use crate::application::policy::user_subject;
use crate::application::test_support::dev_pool;
use crate::domain::file_storage::{
    FileObject, PresignedUrl, StoredFile,
    ports::{FileStorage, FileUpload},
    service::FileStorageService,
};
use crate::{
    Answer, Answers, CampaignPatch, CreateSurveyCommand, MAX_UPLOADED_SURVEY_PHOTOS,
    NormalizedImage, PublicSlug, QuestionId, QuestionKind, ResponseId, ResumeToken,
    SaveMemberResponseCommand, StartMemberResponseCommand, SurveyBranch, SurveyEdge, SurveyEndNode,
    SurveyGraph, SurveyId, SurveyNode, SurveyPatch, SurveyQuestionNode, SurveyRecord,
    SurveyResponseCommand, SurveyWindow, UpdateCampaignCommand, UpdateSurveyCommand,
    UploadMemberResponsePhotoCommand, UserId, Visibility,
};

static PURGES: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Clone, Default)]
pub(super) struct FakeStorage {
    uploaded: Arc<Mutex<Vec<String>>>,
    deleted: Arc<Mutex<Vec<String>>>,
    failing: Arc<Mutex<Option<String>>>,
}

impl FakeStorage {
    fn fail_deleting_under(&self, fragment: String) {
        *self.failing.lock().unwrap() = Some(fragment);
    }

    fn uploaded(&self) -> Vec<String> {
        self.uploaded.lock().unwrap().clone()
    }

    fn deleted(&self) -> Vec<String> {
        self.deleted.lock().unwrap().clone()
    }
}

impl FileStorage for FakeStorage {
    async fn upload(&self, upload: FileUpload) -> Result<StoredFile, CoreError> {
        self.uploaded.lock().unwrap().push(upload.key.clone());
        Ok(StoredFile {
            key: upload.key,
            mime_type: upload.mime_type,
            size_bytes: upload.bytes.len() as u64,
        })
    }

    async fn get(&self, key: &str) -> Result<FileObject, CoreError> {
        Err(CoreError::Internal(format!(
            "no read of {key} in these tests"
        )))
    }

    async fn presigned_get_url(
        &self,
        key: &str,
        _expires_in: std::time::Duration,
    ) -> Result<PresignedUrl, CoreError> {
        Err(CoreError::Internal(format!(
            "no link to {key} in these tests"
        )))
    }

    async fn delete(&self, key: &str) -> Result<(), CoreError> {
        if let Some(fragment) = self.failing.lock().unwrap().as_deref()
            && key.contains(fragment)
        {
            return Err(CoreError::Internal(format!(
                "storage refused to delete {key}"
            )));
        }
        self.deleted.lock().unwrap().push(key.to_owned());
        Ok(())
    }
}

pub(super) fn storage() -> (FakeStorage, FileStorageService<FakeStorage>) {
    let fake = FakeStorage::default();
    let service = FileStorageService::new(fake.clone(), "uploads");
    (fake, service)
}

fn image() -> NormalizedImage {
    NormalizedImage {
        bytes: vec![0xff, 0xd8, 0xff, 0xd9],
        thumbnail: Vec::new(),
        width: 1,
        height: 1,
        content_hash: String::new(),
        taken_at: None,
    }
}

fn qid(n: u128) -> QuestionId {
    QuestionId(Uuid::from_u128(n))
}

fn photo_graph() -> SurveyGraph {
    SurveyGraph::new(
        vec![
            SurveyNode::Question(SurveyQuestionNode {
                id: qid(20),
                kind: QuestionKind::Photo,
                label: "Photo du chantier".to_owned(),
                required: false,
                options: Vec::new(),
                max_photos: None,
                identifier: false,
            }),
            SurveyNode::End(SurveyEndNode {
                id: qid(21),
                message: None,
            }),
        ],
        vec![SurveyEdge {
            from: qid(20),
            to: qid(21),
            branch: SurveyBranch::Next,
        }],
        qid(20),
    )
    .unwrap()
}

fn photo_answers(keys: &[&str]) -> Answers {
    Answers::from([(
        qid(20),
        Answer::Photos(keys.iter().map(|key| (*key).to_owned()).collect()),
    )])
}

async fn photo_survey(
    usecase: &MestierUseCase,
    fixture: &Fixture,
    visibility: Visibility,
    campaign: bool,
) -> SurveyId {
    let id = usecase
        .create_survey(CreateSurveyCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            title: "Photos".to_owned(),
            visibility,
        })
        .await
        .unwrap()
        .id();
    usecase
        .update_survey(UpdateSurveyCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            id,
            patch: SurveyPatch {
                graph: Some(Some(photo_graph())),
                ..SurveyPatch::default()
            },
            campaign_id: campaign.then_some(Some(fixture.campaign_id)),
        })
        .await
        .unwrap();
    usecase
        .open_survey(survey_command(fixture, Subject::system(), id))
        .await
        .unwrap();
    id
}

async fn slug_of(usecase: &MestierUseCase, fixture: &Fixture, id: SurveyId) -> PublicSlug {
    match usecase
        .get_survey(survey_command(fixture, Subject::system(), id))
        .await
        .unwrap()
    {
        SurveyRecord::Opened(survey) => survey.public_slug().cloned().unwrap(),
        other => panic!("expected an opened survey, got {other:?}"),
    }
}

async fn public_response(
    usecase: &MestierUseCase,
    fixture: &Fixture,
) -> (SurveyId, PublicSlug, ResumeToken, ResponseId) {
    let survey_id = photo_survey(usecase, fixture, Visibility::Public, false).await;
    let slug = slug_of(usecase, fixture, survey_id).await;
    let started = usecase.start_public_response(&slug).await.unwrap();
    let token = started.resume_token().cloned().unwrap();
    (survey_id, slug, token, started.id())
}

fn refused_with<T: std::fmt::Debug>(result: Result<T, CoreError>, kind: RefusalKind, reason: &str) {
    match result {
        Err(CoreError::Refused(refusal)) => {
            assert_eq!(refusal.kind, kind, "{refusal:?}");
            assert_eq!(refusal.reason, reason, "{refusal:?}");
        }
        other => panic!("expected the refusal {reason}, got {other:?}"),
    }
}

async fn recorded_photos(response: ResponseId) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM survey.response_photo WHERE response_id = $1")
        .bind(response.0)
        .fetch_one(&dev_pool().await)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_public_photo_is_stored_under_its_response_and_its_key_can_be_answered() {
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let (survey_id, slug, token, response_id) = public_response(&usecase, &fixture).await;

        let key = usecase
            .upload_public_response_photo(&slug, token.clone(), image(), &storage)
            .await
            .unwrap();
        let saved = usecase
            .save_public_response(&slug, token.clone(), photo_answers(&[&key]))
            .await
            .unwrap();

        let prefix = format!(
            "uploads/survey-{}/{}/{}/",
            fixture.organization_id.0, survey_id.0, response_id.0
        );
        assert!(key.starts_with(&prefix), "{key}");
        assert!(key.ends_with(".jpg"), "{key}");
        assert_eq!(fake.uploaded(), vec![key.clone()]);
        assert_eq!(recorded_photos(response_id).await, 1);
        assert_eq!(saved.answers(), &photo_answers(&[&key]));
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_photo_of_another_response_cannot_be_answered() {
    with_fixture(true, async |usecase, fixture| {
        let (_, storage) = storage();
        let (_, slug, token, _) = public_response(&usecase, &fixture).await;
        let other = usecase.start_public_response(&slug).await.unwrap();
        let other_token = other.resume_token().cloned().unwrap();
        let foreign = usecase
            .upload_public_response_photo(&slug, other_token, image(), &storage)
            .await
            .unwrap();

        let refused = usecase
            .save_public_response(&slug, token, photo_answers(&[&foreign]))
            .await;

        refused_with(refused, RefusalKind::Validation, "answer_foreign_photo");
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_public_photo_needs_a_response_in_progress_of_an_opened_survey() {
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let (survey_id, slug, token, _) = public_response(&usecase, &fixture).await;
        let forged = ResumeToken::new("x".repeat(43)).unwrap();
        let unknown = usecase
            .upload_public_response_photo(&slug, forged, image(), &storage)
            .await;
        usecase
            .complete_public_response(&slug, token.clone())
            .await
            .unwrap();
        let completed = usecase
            .upload_public_response_photo(&slug, token, image(), &storage)
            .await;
        let open = usecase.start_public_response(&slug).await.unwrap();
        usecase
            .close_survey(survey_command(&fixture, Subject::system(), survey_id))
            .await
            .unwrap();
        let closed = usecase
            .upload_public_response_photo(
                &slug,
                open.resume_token().cloned().unwrap(),
                image(),
                &storage,
            )
            .await;

        not_found(unknown);
        refused_with(
            completed,
            RefusalKind::Conflict,
            "response_already_completed",
        );
        refused_with(closed, RefusalKind::Conflict, "survey_closed");
        assert!(fake.uploaded().is_empty());
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_twenty_first_photo_of_a_response_is_refused() {
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let (_, slug, token, response_id) = public_response(&usecase, &fixture).await;
        for _ in 0..MAX_UPLOADED_SURVEY_PHOTOS {
            usecase
                .upload_public_response_photo(&slug, token.clone(), image(), &storage)
                .await
                .unwrap();
        }

        let refused = usecase
            .upload_public_response_photo(&slug, token, image(), &storage)
            .await;

        refused_with(refused, RefusalKind::Conflict, "response_photo_cap_reached");
        assert_eq!(MAX_UPLOADED_SURVEY_PHOTOS, 20);
        assert_eq!(fake.uploaded().len(), 20);
        assert_eq!(recorded_photos(response_id).await, 20);
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_photo_is_refused_outside_the_campaign_window() {
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let survey_id = photo_survey(&usecase, &fixture, Visibility::Public, true).await;
        let slug = slug_of(&usecase, &fixture, survey_id).await;
        let started = usecase.start_public_response(&slug).await.unwrap();
        let ended = Utc::now() - Duration::days(1);
        usecase
            .update_campaign(UpdateCampaignCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                id: fixture.campaign_id,
                patch: CampaignPatch {
                    window: Some(SurveyWindow::new(None, Some(ended)).unwrap()),
                    ..CampaignPatch::default()
                },
            })
            .await
            .unwrap();

        let refused = usecase
            .upload_public_response_photo(
                &slug,
                started.resume_token().cloned().unwrap(),
                image(),
                &storage,
            )
            .await;

        refused_with(refused, RefusalKind::Conflict, "campaign_ended");
        assert!(fake.uploaded().is_empty());
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_photo_is_uploaded_only_by_the_member_who_owns_the_response() {
    with_fixture(true, async |usecase, mut fixture| {
        let (fake, storage) = storage();
        let pool = dev_pool().await;
        let colleague = seed_user(&pool).await;
        seed_member(&pool, fixture.organization_id.0, colleague).await;
        fixture.users.push(colleague);
        let survey_id = photo_survey(&usecase, &fixture, Visibility::Members, false).await;
        let response_id = usecase
            .start_member_response(StartMemberResponseCommand {
                actor: fixture.member.clone(),
                organization_id: fixture.organization_id,
                survey_id,
            })
            .await
            .unwrap()
            .response()
            .id();
        let upload = |actor: Subject| {
            usecase.upload_member_response_photo(
                UploadMemberResponsePhotoCommand {
                    actor,
                    organization_id: fixture.organization_id,
                    id: response_id,
                    image: image(),
                },
                &storage,
            )
        };

        let key = upload(fixture.member.clone()).await.unwrap();
        let by_colleague = upload(user_subject(UserId(colleague), Vec::new())).await;
        let by_outsider = upload(fixture.outsider.clone()).await;
        let saved = usecase
            .save_member_response(SaveMemberResponseCommand {
                actor: fixture.member.clone(),
                organization_id: fixture.organization_id,
                id: response_id,
                answers: photo_answers(&[&key]),
            })
            .await;
        usecase
            .complete_member_response(SurveyResponseCommand {
                actor: fixture.member.clone(),
                organization_id: fixture.organization_id,
                id: response_id,
            })
            .await
            .unwrap();
        let after_completion = upload(fixture.member.clone()).await;

        assert!(
            key.starts_with(&format!(
                "uploads/survey-{}/{}/{}/",
                fixture.organization_id.0, survey_id.0, response_id.0
            )),
            "{key}"
        );
        assert!(saved.is_ok(), "{saved:?}");
        not_found(by_colleague);
        assert!(
            matches!(by_outsider, Err(CoreError::Forbidden { .. })),
            "{by_outsider:?}"
        );
        refused_with(
            after_completion,
            RefusalKind::Conflict,
            "response_already_completed",
        );
        assert_eq!(fake.uploaded().len(), 1);
        fixture
    })
    .await;
}

async fn age(response: ResponseId, days: i64) {
    sqlx::query(
        "UPDATE survey.response SET updated_at = now() - make_interval(days => $2) WHERE id = $1",
    )
    .bind(response.0)
    .bind(i32::try_from(days).unwrap())
    .execute(&dev_pool().await)
    .await
    .unwrap();
}

async fn exists(response: ResponseId) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM survey.response WHERE id = $1)")
        .bind(response.0)
        .fetch_one(&dev_pool().await)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_abandoned_response_loses_its_photos_and_a_completed_one_is_never_touched() {
    let _purges = PURGES.lock().await;
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let (_, slug, token, abandoned) = public_response(&usecase, &fixture).await;
        let answered = usecase
            .upload_public_response_photo(&slug, token.clone(), image(), &storage)
            .await
            .unwrap();
        let unreferenced = usecase
            .upload_public_response_photo(&slug, token.clone(), image(), &storage)
            .await
            .unwrap();
        usecase
            .save_public_response(&slug, token, photo_answers(&[&answered]))
            .await
            .unwrap();
        let recent = usecase.start_public_response(&slug).await.unwrap();
        let done = usecase.start_public_response(&slug).await.unwrap();
        let done_token = done.resume_token().cloned().unwrap();
        let kept = usecase
            .upload_public_response_photo(&slug, done_token.clone(), image(), &storage)
            .await
            .unwrap();
        usecase
            .save_public_response(&slug, done_token.clone(), photo_answers(&[&kept]))
            .await
            .unwrap();
        usecase
            .complete_public_response(&slug, done_token)
            .await
            .unwrap();
        age(abandoned, 4000).await;
        age(done.id(), 4000).await;
        age(recent.id(), 29).await;

        for _ in 0..50 {
            let purged = usecase
                .purge_abandoned_survey_photos_in(Utc::now(), &storage)
                .await
                .unwrap();
            if !exists(abandoned).await || purged == 0 {
                break;
            }
        }

        let deleted = fake.deleted();
        assert!(!exists(abandoned).await);
        assert!(deleted.contains(&answered), "{deleted:?}");
        assert!(deleted.contains(&unreferenced), "{deleted:?}");
        assert!(!deleted.contains(&kept), "{deleted:?}");
        assert!(exists(done.id()).await);
        assert!(exists(recent.id()).await);
        assert_eq!(recorded_photos(done.id()).await, 1);
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_purge_without_a_file_storage_does_nothing() {
    with_fixture(true, async |usecase, fixture| {
        assert_eq!(
            usecase.purge_abandoned_survey_photos(Utc::now()).await.ok(),
            Some(0)
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_cheap_photo_checks_refuse_a_capped_response_before_any_normalization() {
    with_fixture(true, async |usecase, fixture| {
        let (_, storage) = storage();
        let (_, slug, token, _) = public_response(&usecase, &fixture).await;
        let open = usecase
            .check_public_response_photo(&slug, token.clone())
            .await;
        for _ in 0..MAX_UPLOADED_SURVEY_PHOTOS {
            usecase
                .upload_public_response_photo(&slug, token.clone(), image(), &storage)
                .await
                .unwrap();
        }

        let capped = usecase.check_public_response_photo(&slug, token).await;
        let forged = usecase
            .check_public_response_photo(&slug, ResumeToken::new("x".repeat(43)).unwrap())
            .await;

        assert!(open.is_ok(), "{open:?}");
        refused_with(capped, RefusalKind::Conflict, "response_photo_cap_reached");
        not_found(forged);
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_member_photo_check_is_reserved_to_the_member_who_owns_the_response() {
    with_fixture(true, async |usecase, mut fixture| {
        let pool = dev_pool().await;
        let colleague = seed_user(&pool).await;
        seed_member(&pool, fixture.organization_id.0, colleague).await;
        fixture.users.push(colleague);
        let survey_id = photo_survey(&usecase, &fixture, Visibility::Members, false).await;
        let response_id = usecase
            .start_member_response(StartMemberResponseCommand {
                actor: fixture.member.clone(),
                organization_id: fixture.organization_id,
                survey_id,
            })
            .await
            .unwrap()
            .response()
            .id();
        let check = |actor: Subject| {
            usecase.check_member_response_photo(SurveyResponseCommand {
                actor,
                organization_id: fixture.organization_id,
                id: response_id,
            })
        };

        let owner = check(fixture.member.clone()).await;
        let by_colleague = check(user_subject(UserId(colleague), Vec::new())).await;

        assert!(owner.is_ok(), "{owner:?}");
        not_found(by_colleague);
        fixture
    })
    .await;
}

async fn forget_upload(key: &str) {
    sqlx::query("DELETE FROM survey.response_photo WHERE key = $1")
        .bind(key)
        .execute(&dev_pool().await)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_key_under_the_response_directory_that_was_never_uploaded_is_refused() {
    with_fixture(true, async |usecase, fixture| {
        let (_, storage) = storage();
        let (survey_id, slug, token, response_id) = public_response(&usecase, &fixture).await;
        let invented = format!(
            "uploads/survey-{}/{}/{}/0195f0a0-0000-7000-8000-00000000abcd.jpg",
            fixture.organization_id.0, survey_id.0, response_id.0
        );
        let on_save = usecase
            .save_public_response(&slug, token.clone(), photo_answers(&[&invented]))
            .await;
        let key = usecase
            .upload_public_response_photo(&slug, token.clone(), image(), &storage)
            .await
            .unwrap();
        usecase
            .save_public_response(&slug, token.clone(), photo_answers(&[&key]))
            .await
            .unwrap();
        forget_upload(&key).await;

        let on_complete = usecase.complete_public_response(&slug, token).await;

        refused_with(on_save, RefusalKind::Validation, "answer_foreign_photo");
        refused_with(on_complete, RefusalKind::Validation, "answer_foreign_photo");
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_purge_deletes_only_the_recorded_uploads() {
    let _purges = PURGES.lock().await;
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let (_, slug, token, abandoned) = public_response(&usecase, &fixture).await;
        let answered = usecase
            .upload_public_response_photo(&slug, token.clone(), image(), &storage)
            .await
            .unwrap();
        let recorded = usecase
            .upload_public_response_photo(&slug, token.clone(), image(), &storage)
            .await
            .unwrap();
        usecase
            .save_public_response(&slug, token, photo_answers(&[&answered]))
            .await
            .unwrap();
        forget_upload(&answered).await;
        age(abandoned, 4000).await;

        usecase
            .purge_abandoned_survey_photos_in(Utc::now(), &storage)
            .await
            .unwrap();

        let deleted = fake.deleted();
        assert!(!exists(abandoned).await);
        assert!(deleted.contains(&recorded), "{deleted:?}");
        assert!(!deleted.contains(&answered), "{deleted:?}");
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_storage_failure_on_one_abandoned_response_does_not_block_the_others_of_the_run() {
    let _purges = PURGES.lock().await;
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let (_, slug, token, poisoned) = public_response(&usecase, &fixture).await;
        usecase
            .upload_public_response_photo(&slug, token, image(), &storage)
            .await
            .unwrap();
        let healthy = usecase.start_public_response(&slug).await.unwrap();
        let healthy_key = usecase
            .upload_public_response_photo(
                &slug,
                healthy.resume_token().cloned().unwrap(),
                image(),
                &storage,
            )
            .await
            .unwrap();
        age(poisoned, 9000).await;
        age(healthy.id(), 8999).await;
        fake.fail_deleting_under(poisoned.0.to_string());

        let purged = usecase
            .purge_abandoned_survey_photos_paged(Utc::now(), &storage, 1)
            .await
            .unwrap();

        assert!(purged >= 1);
        assert!(exists(poisoned).await);
        assert_eq!(recorded_photos(poisoned).await, 1);
        assert!(!exists(healthy.id()).await);
        assert!(fake.deleted().contains(&healthy_key));
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn deleting_a_response_removes_its_photos_from_storage_first() {
    with_fixture(true, async |usecase, fixture| {
        let (fake, storage) = storage();
        let (_, slug, token, response_id) = public_response(&usecase, &fixture).await;
        let first = usecase
            .upload_public_response_photo(&slug, token.clone(), image(), &storage)
            .await
            .unwrap();
        let second = usecase
            .upload_public_response_photo(&slug, token, image(), &storage)
            .await
            .unwrap();
        let command = || SurveyResponseCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            id: response_id,
        };
        fake.fail_deleting_under(second.clone());

        let failed = usecase.delete_survey_response(command(), &storage).await;
        let kept = exists(response_id).await;
        fake.fail_deleting_under("never-matches-a-key".to_owned());
        usecase
            .delete_survey_response(command(), &storage)
            .await
            .unwrap();

        assert!(failed.is_err(), "{failed:?}");
        assert!(kept);
        assert!(!exists(response_id).await);
        assert_eq!(recorded_photos(response_id).await, 0);
        let deleted = fake.deleted();
        assert!(deleted.contains(&first), "{deleted:?}");
        assert!(deleted.contains(&second), "{deleted:?}");
        fixture
    })
    .await;
}
