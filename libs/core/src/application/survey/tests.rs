use authz::Subject;
use common::{CoreError, OrganizationId, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::policy::user_subject;
use crate::application::test_support::{dev_pool, now_storable, purge};
use crate::application::{MestierUseCase, default_authorizer};
use crate::infrastructure::realtime::EventHub;
use crate::{
    Answer, Answers, CampaignCommand, CampaignId, CampaignPatch, CampaignStatus,
    CreateCampaignCommand, CreateSurveyCommand, ListCampaignSurveysCommand, ListCampaignsCommand,
    ListSurveyResponsesCommand, ListSurveysCommand, Metadata, PublicSlug, QuestionId, QuestionKind,
    ResponseId, ResumeToken, SaveMemberResponseCommand, SetSurveyCampaignsCommand,
    StartMemberResponseCommand, SurveyBranch, SurveyCommand, SurveyEdge, SurveyEndNode,
    SurveyGraph, SurveyGraphVersionCommand, SurveyId, SurveyNode, SurveyPatch, SurveyQuestionNode,
    SurveyResponseCommand, SurveyWindow, UpdateCampaignCommand, UpdateSurveyCommand, UserId,
    Visibility,
};

pub(super) struct Fixture {
    pub(super) organization_id: OrganizationId,
    pub(super) users: Vec<Uuid>,
    organizations: Vec<Uuid>,
    pub(super) member: Subject,
    pub(super) outsider: Subject,
    survey_id: SurveyId,
    pub(super) campaign_id: CampaignId,
}

fn make_usecase(pool: PgPool) -> MestierUseCase {
    MestierUseCase::new(pool, default_authorizer(), EventHub::new())
}

fn qid(n: u128) -> QuestionId {
    QuestionId(Uuid::from_u128(n))
}

fn graph() -> SurveyGraph {
    SurveyGraph::new(
        vec![
            SurveyNode::Question(SurveyQuestionNode {
                id: qid(10),
                kind: QuestionKind::Text,
                label: "Nom".to_owned(),
                required: true,
                options: Vec::new(),
                max_photos: None,
                identifier: false,
            }),
            SurveyNode::End(SurveyEndNode {
                id: qid(11),
                message: Some("Merci".to_owned()),
            }),
        ],
        vec![SurveyEdge {
            from: qid(10),
            to: qid(11),
            branch: SurveyBranch::Next,
        }],
        qid(10),
    )
    .unwrap()
}

fn answers(name: &str) -> Answers {
    Answers::from([(qid(10), Answer::Text(name.to_owned()))])
}

pub(super) async fn seed_user(pool: &PgPool) -> Uuid {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Survey', $4)",
    )
    .bind(id)
    .bind(format!("survey-{id}@example.com"))
    .bind(format!("survey-{id}"))
    .bind(format!("sub-survey-{id}"))
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn seed_organization(pool: &PgPool, owner: Uuid, flag: bool) -> Uuid {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Survey Org', $2, $3)",
    )
    .bind(id)
    .bind(format!("survey-org-{id}"))
    .bind(owner)
    .execute(pool)
    .await
    .unwrap();
    if flag {
        set_flag(pool, id, true).await;
    }
    id
}

async fn set_flag(pool: &PgPool, organization: Uuid, enabled: bool) {
    let statement = if enabled {
        "INSERT INTO organization_feature_flags (org_id, flag) VALUES ($1, 'surveys')"
    } else {
        "DELETE FROM organization_feature_flags WHERE org_id = $1 AND flag = 'surveys'"
    };
    sqlx::query(statement)
        .bind(organization)
        .execute(pool)
        .await
        .unwrap();
}

pub(super) async fn seed_member(pool: &PgPool, organization: Uuid, user: Uuid) {
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, 'Claire')",
    )
    .bind(generate_uuid_v7())
    .bind(organization)
    .bind(user)
    .execute(pool)
    .await
    .unwrap();
}

async fn seed(pool: &PgPool, usecase: &MestierUseCase, flag: bool) -> Fixture {
    let owner = seed_user(pool).await;
    let member = seed_user(pool).await;
    let outsider = seed_user(pool).await;
    let organization = seed_organization(pool, owner, flag).await;
    let other = seed_organization(pool, owner, true).await;
    seed_member(pool, organization, member).await;
    seed_member(pool, other, outsider).await;
    let organization_id = OrganizationId(organization);
    let mut fixture = Fixture {
        organization_id,
        users: vec![owner, member, outsider],
        organizations: vec![organization, other],
        member: user_subject(UserId(member), Vec::new()),
        outsider: user_subject(UserId(outsider), Vec::new()),
        survey_id: SurveyId(generate_uuid_v7()),
        campaign_id: CampaignId(generate_uuid_v7()),
    };
    if flag {
        fixture.survey_id = draft(usecase, &fixture, Visibility::Members).await;
        fixture.campaign_id = usecase
            .create_campaign(CreateCampaignCommand {
                actor: Subject::system(),
                organization_id,
                name: "Printemps".to_owned(),
                window: SurveyWindow::new(None, None).unwrap(),
                metadata: Metadata::default(),
            })
            .await
            .unwrap()
            .id();
    }
    fixture
}

async fn draft(usecase: &MestierUseCase, fixture: &Fixture, visibility: Visibility) -> SurveyId {
    let created = usecase
        .create_survey(CreateSurveyCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            title: "Satisfaction".to_owned(),
            visibility,
            campaign_ids: Vec::new(),
        })
        .await
        .unwrap();
    usecase
        .update_survey(UpdateSurveyCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            id: created.id(),
            patch: SurveyPatch {
                graph: Some(Some(graph())),
                ..SurveyPatch::default()
            },
            campaign_ids: None,
        })
        .await
        .unwrap();
    created.id()
}

async fn opened(usecase: &MestierUseCase, fixture: &Fixture, visibility: Visibility) -> SurveyId {
    let id = draft(usecase, fixture, visibility).await;
    usecase
        .open_survey(survey_command(fixture, Subject::system(), id))
        .await
        .unwrap();
    id
}

pub(super) fn survey_command(fixture: &Fixture, actor: Subject, id: SurveyId) -> SurveyCommand {
    SurveyCommand {
        actor,
        organization_id: fixture.organization_id,
        id,
    }
}

fn campaign_command(fixture: &Fixture) -> CampaignCommand {
    CampaignCommand {
        actor: fixture.member.clone(),
        organization_id: fixture.organization_id,
        id: fixture.campaign_id,
    }
}

fn response_command(fixture: &Fixture, actor: Subject, id: ResponseId) -> SurveyResponseCommand {
    SurveyResponseCommand {
        actor,
        organization_id: fixture.organization_id,
        id,
    }
}

async fn cleanup(pool: &PgPool, fixture: &Fixture) {
    for organization in &fixture.organizations {
        for statement in [
            "DELETE FROM survey.response WHERE org_id = $1",
            "DELETE FROM survey.survey WHERE org_id = $1",
            "DELETE FROM survey.campaign WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, *organization).await;
        }
    }
    for user in &fixture.users {
        purge(pool, "DELETE FROM users WHERE id = $1", *user).await;
    }
}

fn forbidden<T: std::fmt::Debug>(result: Result<T, CoreError>) {
    assert!(
        matches!(result, Err(CoreError::Forbidden { .. })),
        "expected a refusal, got {result:?}"
    );
}

pub(super) fn not_found<T: std::fmt::Debug>(result: Result<T, CoreError>) {
    assert!(
        matches!(result, Err(CoreError::NotFound)),
        "expected not found, got {result:?}"
    );
}

pub(super) async fn with_fixture<F>(flag: bool, check: F)
where
    F: AsyncFnOnce(MestierUseCase, Fixture) -> Fixture,
{
    let pool = dev_pool().await;
    let usecase = make_usecase(pool.clone());
    let fixture = seed(&pool, &usecase, flag).await;
    let fixture = check(usecase, fixture).await;
    cleanup(&pool, &fixture).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn create_survey_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .create_survey(CreateSurveyCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    title: "Refus".to_owned(),
                    visibility: Visibility::Members,
                    campaign_ids: Vec::new(),
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn update_survey_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .update_survey(UpdateSurveyCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    id: fixture.survey_id,
                    patch: SurveyPatch::default(),
                    campaign_ids: None,
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn open_survey_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .open_survey(survey_command(
                    &fixture,
                    fixture.member.clone(),
                    fixture.survey_id,
                ))
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn close_survey_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .close_survey(survey_command(
                    &fixture,
                    fixture.member.clone(),
                    fixture.survey_id,
                ))
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn duplicate_survey_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .duplicate_survey(survey_command(
                    &fixture,
                    fixture.member.clone(),
                    fixture.survey_id,
                ))
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn delete_survey_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .delete_survey(survey_command(
                    &fixture,
                    fixture.member.clone(),
                    fixture.survey_id,
                ))
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_surveys_refuses_a_member_without_survey_view() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .list_surveys(ListSurveysCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    campaign_id: None,
                    limit: 10,
                    offset: 0,
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn get_survey_refuses_a_member_without_survey_view() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .get_survey(survey_command(
                    &fixture,
                    fixture.member.clone(),
                    fixture.survey_id,
                ))
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn create_campaign_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .create_campaign(CreateCampaignCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    name: "Refus".to_owned(),
                    window: SurveyWindow::new(None, None).unwrap(),
                    metadata: Metadata::default(),
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn update_campaign_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .update_campaign(UpdateCampaignCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    id: fixture.campaign_id,
                    patch: CampaignPatch::default(),
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn delete_campaign_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(usecase.delete_campaign(campaign_command(&fixture)).await);
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn get_campaign_reads_a_campaign_of_its_organization_only() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(usecase.get_campaign(campaign_command(&fixture)).await);
        let read = usecase
            .get_campaign(CampaignCommand {
                actor: Subject::system(),
                ..campaign_command(&fixture)
            })
            .await
            .unwrap();
        assert_eq!(read.id(), fixture.campaign_id);
        assert_eq!(read.name(), "Printemps");
        not_found(
            usecase
                .get_campaign(CampaignCommand {
                    actor: Subject::system(),
                    organization_id: OrganizationId(fixture.organizations[1]),
                    id: fixture.campaign_id,
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_campaigns_refuses_a_member_without_survey_view() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .list_campaigns(ListCampaignsCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    limit: 10,
                    offset: 0,
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_survey_responses_refuses_a_member_without_survey_view() {
    with_fixture(true, async |usecase, fixture| {
        forbidden(
            usecase
                .list_survey_responses(ListSurveyResponsesCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    survey_id: fixture.survey_id,
                    limit: 10,
                    offset: 0,
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn get_survey_response_refuses_a_member_without_survey_view() {
    with_fixture(true, async |usecase, fixture| {
        let id = ResponseId(generate_uuid_v7());
        forbidden(
            usecase
                .get_survey_response(response_command(&fixture, fixture.member.clone(), id))
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn delete_survey_response_refuses_a_member_without_survey_manage() {
    with_fixture(true, async |usecase, fixture| {
        let id = ResponseId(generate_uuid_v7());
        forbidden(
            usecase
                .delete_survey_response(
                    response_command(&fixture, fixture.member.clone(), id),
                    &super::photos_tests::storage().1,
                )
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_use_cases_answer_not_found_when_the_flag_is_off() {
    with_fixture(false, async |usecase, fixture| {
        not_found(
            usecase
                .create_survey(CreateSurveyCommand {
                    actor: Subject::system(),
                    organization_id: fixture.organization_id,
                    title: "Sans flag".to_owned(),
                    visibility: Visibility::Members,
                    campaign_ids: Vec::new(),
                })
                .await,
        );
        not_found(
            usecase
                .list_surveys(ListSurveysCommand {
                    actor: Subject::system(),
                    organization_id: fixture.organization_id,
                    campaign_id: None,
                    limit: 10,
                    offset: 0,
                })
                .await,
        );
        not_found(
            usecase
                .start_member_response(StartMemberResponseCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    survey_id: fixture.survey_id,
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_of_another_organization_cannot_answer_a_members_survey() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Members).await;

        forbidden(
            usecase
                .start_member_response(StartMemberResponseCommand {
                    actor: fixture.outsider.clone(),
                    organization_id: fixture.organization_id,
                    survey_id,
                })
                .await,
        );
        forbidden(
            usecase
                .start_member_response(StartMemberResponseCommand {
                    actor: Subject::system(),
                    organization_id: fixture.organization_id,
                    survey_id,
                })
                .await,
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_answers_a_members_survey_once_without_any_survey_permission() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Members).await;
        let start = || StartMemberResponseCommand {
            actor: fixture.member.clone(),
            organization_id: fixture.organization_id,
            survey_id,
        };

        let created = usecase.start_member_response(start()).await.unwrap();
        let again = usecase.start_member_response(start()).await.unwrap();
        assert!(!created.is_resumed());
        assert!(again.is_resumed());
        let started = created.into_response();
        let resumed = again.into_response();
        usecase
            .save_member_response(SaveMemberResponseCommand {
                actor: fixture.member.clone(),
                organization_id: fixture.organization_id,
                id: started.id(),
                answers: answers("Claire"),
            })
            .await
            .unwrap();
        let completed = usecase
            .complete_member_response(response_command(
                &fixture,
                fixture.member.clone(),
                started.id(),
            ))
            .await
            .unwrap();
        let again = usecase.start_member_response(start()).await;

        assert_eq!(resumed.id(), started.id());
        assert_eq!(completed.answers(), &answers("Claire"));
        assert!(
            matches!(again, Err(CoreError::Refused(ref refusal)) if refusal.reason == "respondent_already_answered"),
            "{again:?}"
        );
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_public_survey_is_answered_without_an_account() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Public).await;
        let survey = usecase
            .get_survey(survey_command(&fixture, Subject::system(), survey_id))
            .await
            .unwrap();
        let slug = match survey {
            crate::SurveyRecord::Opened(survey) => survey.public_slug().cloned().unwrap(),
            other => panic!("expected an opened survey, got {other:?}"),
        };

        let read = usecase.open_public_survey(&slug).await.unwrap();
        let started = usecase.start_public_response(&slug).await.unwrap();
        let token = started.resume_token().cloned().unwrap();
        let resumed = usecase
            .resume_public_response(&slug, token.clone())
            .await
            .unwrap();
        let another = usecase.start_public_response(&slug).await.unwrap();
        let forged = ResumeToken::new("x".repeat(43)).unwrap();
        let unknown = usecase.resume_public_response(&slug, forged).await;
        usecase
            .save_public_response(&slug, token.clone(), answers("Claire"))
            .await
            .unwrap();
        let completed = usecase
            .complete_public_response(&slug, token)
            .await
            .unwrap();
        let (responses, total) = usecase
            .list_survey_responses(ListSurveyResponsesCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                survey_id,
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();

        assert_eq!(read.id(), survey_id);
        assert!(slug.as_str().len() >= 16);
        assert!(token_is_long(started.resume_token()));
        assert_eq!(resumed.id(), started.id());
        assert_ne!(another.id(), started.id());
        assert_ne!(another.resume_token(), started.resume_token());
        not_found(unknown);
        assert_eq!(completed.member_id(), None);
        assert_eq!(total, 2);
        assert!(
            responses
                .iter()
                .any(|response| response.id() == started.id())
        );
        fixture
    })
    .await;
}

fn token_is_long(token: Option<&ResumeToken>) -> bool {
    token.is_some_and(|token| token.as_str().len() >= 32)
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_public_survey_of_an_organization_without_the_flag_answers_like_an_unknown_slug() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Public).await;
        let slug = match usecase
            .get_survey(survey_command(&fixture, Subject::system(), survey_id))
            .await
            .unwrap()
        {
            crate::SurveyRecord::Opened(survey) => survey.public_slug().cloned().unwrap(),
            other => panic!("expected an opened survey, got {other:?}"),
        };
        let pool = dev_pool().await;
        set_flag(&pool, fixture.organization_id.0, false).await;
        let unknown = PublicSlug::new("inconnu-0000000000".to_owned()).unwrap();

        let disabled = usecase
            .open_public_survey(&slug)
            .await
            .map(|survey| survey.id());
        let missing = usecase
            .open_public_survey(&unknown)
            .await
            .map(|survey| survey.id());
        let start = usecase.start_public_response(&slug).await.map(|r| r.id());

        assert_eq!(format!("{disabled:?}"), format!("{missing:?}"));
        not_found(disabled);
        not_found(start);
        fixture
    })
    .await;
}

async fn scheduled_campaign(
    usecase: &MestierUseCase,
    fixture: &Fixture,
    starts_at: Option<chrono::DateTime<chrono::Utc>>,
    ends_at: Option<chrono::DateTime<chrono::Utc>>,
) -> CampaignId {
    usecase
        .create_campaign(CreateCampaignCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            name: "Horloge".to_owned(),
            window: SurveyWindow::new(starts_at, ends_at).unwrap(),
            metadata: Metadata::default(),
        })
        .await
        .unwrap()
        .id()
}

async fn campaign_events(pool: &PgPool, fixture: &Fixture, id: CampaignId) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT name FROM automation.event WHERE org_id = $1 AND payload->>'campaign_id' = $2 ORDER BY id",
    )
    .bind(fixture.organization_id.0)
    .bind(id.0.to_string())
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn campaign_status(
    usecase: &MestierUseCase,
    fixture: &Fixture,
    id: CampaignId,
) -> CampaignStatus {
    let (campaigns, _) = usecase
        .list_campaigns(ListCampaignsCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            limit: 100,
            offset: 0,
        })
        .await
        .unwrap();
    campaigns
        .into_iter()
        .find(|campaign| campaign.id() == id)
        .map(|campaign| campaign.status())
        .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_survey_campaign_clock_emits_each_step_once() {
    with_fixture(true, async |usecase, fixture| {
        let pool = dev_pool().await;
        let now = now_storable();
        let hours = chrono::Duration::hours;
        let opening = scheduled_campaign(
            &usecase,
            &fixture,
            Some(now + hours(1)),
            Some(now + hours(3)),
        )
        .await;
        let skipped = scheduled_campaign(
            &usecase,
            &fixture,
            Some(now + hours(1)),
            Some(now + hours(2)),
        )
        .await;
        let windowless = fixture.campaign_id;
        let windowless_before = campaign_events(&pool, &fixture, windowless).await;

        let quiet = campaign_events(&pool, &fixture, opening).await;
        usecase
            .advance_survey_campaigns(now + hours(2) + hours(1) / 2)
            .await
            .unwrap();
        usecase
            .advance_survey_campaigns(now + hours(2) + hours(1) / 2)
            .await
            .unwrap();
        let opened = campaign_events(&pool, &fixture, opening).await;
        let both = campaign_events(&pool, &fixture, skipped).await;
        let running = campaign_status(&usecase, &fixture, opening).await;
        usecase
            .advance_survey_campaigns(now + hours(4))
            .await
            .unwrap();
        let ended = campaign_events(&pool, &fixture, opening).await;
        let windowless_after = campaign_events(&pool, &fixture, windowless).await;

        assert!(quiet.is_empty());
        assert_eq!(opened, vec!["survey_campaign.started"]);
        assert_eq!(running, CampaignStatus::Running);
        assert_eq!(
            both,
            vec!["survey_campaign.started", "survey_campaign.ended"]
        );
        assert_eq!(
            ended,
            vec!["survey_campaign.started", "survey_campaign.ended"]
        );
        assert_eq!(
            campaign_status(&usecase, &fixture, skipped).await,
            CampaignStatus::Ended
        );
        assert_eq!(windowless_after, windowless_before);
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_reads_an_opened_members_survey_without_any_survey_permission() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Members).await;

        let view = usecase
            .open_member_survey(survey_command(&fixture, fixture.member.clone(), survey_id))
            .await
            .unwrap();

        assert_eq!(view.id(), survey_id);
        assert_eq!(view.visibility(), Visibility::Members);
        assert_eq!(view.public_slug(), None);
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_survey_read_is_refused_outside_an_opened_members_survey() {
    with_fixture(true, async |usecase, fixture| {
        let members = opened(&usecase, &fixture, Visibility::Members).await;
        let public = opened(&usecase, &fixture, Visibility::Public).await;
        let read = |actor: Subject, id: SurveyId| {
            usecase.open_member_survey(survey_command(&fixture, actor, id))
        };

        forbidden(read(fixture.outsider.clone(), members).await);
        not_found(read(fixture.member.clone(), public).await);
        not_found(read(fixture.member.clone(), fixture.survey_id).await);
        not_found(read(fixture.member.clone(), SurveyId(generate_uuid_v7())).await);
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_survey_read_answers_not_found_when_the_flag_is_off() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Members).await;
        let pool = dev_pool().await;
        set_flag(&pool, fixture.organization_id.0, false).await;

        not_found(
            usecase
                .open_member_survey(survey_command(&fixture, fixture.member.clone(), survey_id))
                .await,
        );
        fixture
    })
    .await;
}

struct EnabledFlags;

impl crate::domain::feature_flag::ports::FeatureFlagRepository for EnabledFlags {
    async fn list_enabled(
        &mut self,
        _organization_id: OrganizationId,
    ) -> Result<Vec<crate::FeatureActivation>, CoreError> {
        Ok(Vec::new())
    }

    async fn is_enabled(
        &mut self,
        _organization_id: OrganizationId,
        _flag: crate::FeatureFlag,
    ) -> Result<bool, CoreError> {
        Ok(true)
    }

    async fn enable(
        &mut self,
        _organization_id: OrganizationId,
        _flag: crate::FeatureFlag,
        _by: UserId,
    ) -> Result<bool, CoreError> {
        Ok(true)
    }

    async fn disable(
        &mut self,
        _organization_id: OrganizationId,
        _flag: crate::FeatureFlag,
    ) -> Result<bool, CoreError> {
        Ok(true)
    }
}

fn surveys_found_by_slug(
    record: crate::SurveyRecord,
) -> crate::domain::survey::ports::MockSurveyRepository {
    let mut surveys = crate::domain::survey::ports::MockSurveyRepository::new();
    surveys.expect_find_by_public_slug().returning(move |_| {
        let found = record.clone();
        Box::pin(async move { Ok(Some(found)) })
    });
    surveys
}

fn opened_members_survey() -> crate::SurveyRecord {
    let mut survey = crate::Survey::new(
        SurveyId(Uuid::from_u128(1)),
        OrganizationId(Uuid::from_u128(2)),
        "Satisfaction".to_owned(),
        Visibility::Members,
    )
    .unwrap();
    survey.edit_graph(Some(graph()));
    survey.open(chrono::Utc::now(), None).unwrap().into()
}

#[tokio::test]
async fn the_public_use_cases_do_not_find_an_opened_members_survey() {
    let slug = PublicSlug::new("abcdefgh".to_owned()).unwrap();
    let refused = super::public_survey(
        &mut surveys_found_by_slug(opened_members_survey()),
        &mut EnabledFlags,
        &slug,
    )
    .await;
    assert!(matches!(refused, Err(CoreError::NotFound)), "{refused:?}");
}

fn second_graph() -> SurveyGraph {
    SurveyGraph::new(
        vec![
            SurveyNode::Question(SurveyQuestionNode {
                id: qid(20),
                kind: QuestionKind::Text,
                label: "Prénom".to_owned(),
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

fn graph_version_command(
    fixture: &Fixture,
    actor: Subject,
    id: SurveyId,
    version: u32,
) -> SurveyGraphVersionCommand {
    SurveyGraphVersionCommand {
        actor,
        organization_id: fixture.organization_id,
        id,
        version: crate::SurveyGraphVersion::new(version).unwrap(),
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_opened_survey_is_edited_versioned_and_attached_to_campaigns() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Members).await;
        let started = usecase
            .start_member_response(StartMemberResponseCommand {
                actor: fixture.member.clone(),
                organization_id: fixture.organization_id,
                survey_id,
            })
            .await
            .unwrap()
            .into_response();
        usecase
            .save_member_response(SaveMemberResponseCommand {
                actor: fixture.member.clone(),
                organization_id: fixture.organization_id,
                id: started.id(),
                answers: answers("Claire"),
            })
            .await
            .unwrap();
        usecase
            .complete_member_response(response_command(
                &fixture,
                fixture.member.clone(),
                started.id(),
            ))
            .await
            .unwrap();

        let edited = usecase
            .update_survey(UpdateSurveyCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                id: survey_id,
                patch: SurveyPatch {
                    title: Some("Bilan".to_owned()),
                    graph: Some(Some(second_graph())),
                    ..SurveyPatch::default()
                },
                campaign_ids: Some(vec![fixture.campaign_id]),
            })
            .await
            .unwrap();
        assert!(matches!(edited, crate::SurveyRecord::Opened(_)));
        assert_eq!(edited.graph_version().map(|version| version.get()), Some(2));
        assert_eq!(edited.campaign_ids(), &[fixture.campaign_id]);

        let former = usecase
            .get_survey_graph_version(graph_version_command(&fixture, Subject::system(), survey_id, 1))
            .await
            .unwrap();
        assert_eq!(former.graph(), &graph());
        let current = usecase
            .get_survey_graph_version(graph_version_command(&fixture, Subject::system(), survey_id, 2))
            .await
            .unwrap();
        assert_eq!(current.graph(), &second_graph());
        not_found(
            usecase
                .get_survey_graph_version(graph_version_command(
                    &fixture,
                    Subject::system(),
                    survey_id,
                    3,
                ))
                .await,
        );
        forbidden(
            usecase
                .get_survey_graph_version(graph_version_command(
                    &fixture,
                    fixture.member.clone(),
                    survey_id,
                    1,
                ))
                .await,
        );

        let listed = usecase
            .list_completed_survey_responses(ListSurveyResponsesCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                survey_id,
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!(listed.responses.len(), 1);
        assert_eq!(listed.responses[0].graph_version().get(), 1);
        assert_eq!(listed.graph_of(&listed.responses[0]), Ok(&graph()));

        let (held, held_total) = usecase
            .list_campaign_surveys(ListCampaignSurveysCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                id: fixture.campaign_id,
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!((held.len(), held_total), (1, 1));
        assert_eq!(held[0].id, survey_id);
        assert_eq!(held[0].title, "Bilan");
        let (filtered, total) = usecase
            .list_surveys(ListSurveysCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                campaign_id: Some(fixture.campaign_id),
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!((filtered.len(), total), (1, 1));
        forbidden(
            usecase
                .list_campaign_surveys(ListCampaignSurveysCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    id: fixture.campaign_id,
                    limit: 10,
                    offset: 0,
                })
                .await,
        );

        let cleared = usecase
            .set_survey_campaigns(SetSurveyCampaignsCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                id: survey_id,
                campaign_ids: Vec::new(),
            })
            .await
            .unwrap();
        assert!(cleared.campaign_ids().is_empty());
        forbidden(
            usecase
                .set_survey_campaigns(SetSurveyCampaignsCommand {
                    actor: fixture.member.clone(),
                    organization_id: fixture.organization_id,
                    id: survey_id,
                    campaign_ids: Vec::new(),
                })
                .await,
        );
        let frozen = usecase
            .update_survey(UpdateSurveyCommand {
                actor: Subject::system(),
                organization_id: fixture.organization_id,
                id: survey_id,
                patch: SurveyPatch {
                    visibility: Some(Visibility::Public),
                    ..SurveyPatch::default()
                },
                campaign_ids: None,
            })
            .await;
        assert!(
            matches!(frozen, Err(CoreError::Refused(ref refusal)) if refusal.reason == "survey_visibility_frozen"),
            "{frozen:?}"
        );
        fixture
    })
    .await;
}

fn relabelled(label: &str) -> SurveyGraph {
    SurveyGraph::new(
        vec![
            SurveyNode::Question(SurveyQuestionNode {
                id: qid(10),
                kind: QuestionKind::Text,
                label: label.to_owned(),
                required: true,
                options: Vec::new(),
                max_photos: None,
                identifier: false,
            }),
            SurveyNode::End(SurveyEndNode {
                id: qid(11),
                message: None,
            }),
        ],
        vec![SurveyEdge {
            from: qid(10),
            to: qid(11),
            branch: SurveyBranch::Next,
        }],
        qid(10),
    )
    .unwrap()
}

async fn respond(
    usecase: &MestierUseCase,
    fixture: &Fixture,
    survey_id: SurveyId,
    name: &str,
) -> ResponseId {
    let started = usecase
        .start_member_response(StartMemberResponseCommand {
            actor: fixture.member.clone(),
            organization_id: fixture.organization_id,
            survey_id,
        })
        .await
        .unwrap()
        .into_response();
    usecase
        .save_member_response(SaveMemberResponseCommand {
            actor: fixture.member.clone(),
            organization_id: fixture.organization_id,
            id: started.id(),
            answers: answers(name),
        })
        .await
        .unwrap();
    usecase
        .complete_member_response(response_command(
            fixture,
            fixture.member.clone(),
            started.id(),
        ))
        .await
        .unwrap();
    started.id()
}

async fn relabel(usecase: &MestierUseCase, fixture: &Fixture, survey_id: SurveyId, label: &str) {
    usecase
        .update_survey(UpdateSurveyCommand {
            actor: Subject::system(),
            organization_id: fixture.organization_id,
            id: survey_id,
            patch: SurveyPatch {
                graph: Some(Some(relabelled(label))),
                ..SurveyPatch::default()
            },
            campaign_ids: None,
        })
        .await
        .unwrap();
}

fn responses_command(fixture: &Fixture, survey_id: SurveyId) -> ListSurveyResponsesCommand {
    ListSurveyResponsesCommand {
        actor: Subject::system(),
        organization_id: fixture.organization_id,
        survey_id,
        limit: 10,
        offset: 0,
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_deleted_test_response_leaves_every_graph_version_readable() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Members).await;
        let (_, storage) = super::photos_tests::storage();

        let tried = respond(&usecase, &fixture, survey_id, "Essai").await;
        usecase
            .delete_survey_response(
                response_command(&fixture, Subject::system(), tried),
                &storage,
            )
            .await
            .unwrap();
        relabel(&usecase, &fixture, survey_id, "Prénom").await;
        let kept = respond(&usecase, &fixture, survey_id, "Claire").await;
        relabel(&usecase, &fixture, survey_id, "Âge").await;

        for (number, expected) in [
            (1, graph()),
            (2, relabelled("Prénom")),
            (3, relabelled("Âge")),
        ] {
            let read = usecase
                .get_survey_graph_version(graph_version_command(
                    &fixture,
                    Subject::system(),
                    survey_id,
                    number,
                ))
                .await
                .unwrap();
            assert_eq!(read.graph(), &expected, "version {number}");
        }
        let listed = usecase
            .list_completed_survey_responses(responses_command(&fixture, survey_id))
            .await
            .unwrap();
        assert_eq!(listed.responses.len(), 1);
        assert_eq!(listed.responses[0].id(), kept);
        assert_eq!(listed.responses[0].graph_version().get(), 2);
        assert_eq!(
            listed.graph_of(&listed.responses[0]),
            Ok(&relabelled("Prénom"))
        );
        let (records, total) = usecase
            .list_survey_responses(responses_command(&fixture, survey_id))
            .await
            .unwrap();
        assert_eq!((records.len(), total), (1, 1));
        fixture
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_completed_response_whose_version_row_is_missing_is_reported() {
    with_fixture(true, async |usecase, fixture| {
        let survey_id = opened(&usecase, &fixture, Visibility::Members).await;
        respond(&usecase, &fixture, survey_id, "Claire").await;
        relabel(&usecase, &fixture, survey_id, "Prénom").await;
        sqlx::query("DELETE FROM survey.survey_version WHERE survey_id = $1")
            .bind(survey_id.0)
            .execute(&dev_pool().await)
            .await
            .unwrap();

        let listed = usecase
            .list_completed_survey_responses(responses_command(&fixture, survey_id))
            .await
            .unwrap();
        assert_eq!(
            listed.graph_of(&listed.responses[0]),
            Err(crate::SurveyError::GraphVersionNotFound)
        );
        not_found(
            usecase
                .list_survey_responses(responses_command(&fixture, survey_id))
                .await,
        );
        fixture
    })
    .await;
}

mod versions {
    use std::sync::{Arc, Mutex};

    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::super::{former_graphs, require_graphs};
    use crate::domain::survey::ports::MockSurveyRepository;
    use crate::{
        OrganizationId, Survey, SurveyError, SurveyGraphVersion, SurveyId, SurveyRecord, Visibility,
    };

    fn version(number: u32) -> SurveyGraphVersion {
        SurveyGraphVersion::new(number).unwrap()
    }

    fn at_version_three() -> SurveyRecord {
        let mut survey = Survey::new(
            SurveyId(Uuid::from_u128(1)),
            OrganizationId(Uuid::from_u128(2)),
            "Satisfaction".to_owned(),
            Visibility::Members,
        )
        .unwrap();
        survey.edit_graph(Some(super::graph()));
        let mut opened = survey
            .open(Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap(), None)
            .unwrap();
        opened.revise_graph(super::relabelled("Prénom")).unwrap();
        opened.revise_graph(super::relabelled("Âge")).unwrap();
        opened.into()
    }

    #[tokio::test]
    async fn each_former_version_is_read_once_even_when_it_is_missing() {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let mut surveys = MockSurveyRepository::new();
        let log = Arc::clone(&asked);
        surveys
            .expect_find_graph_version()
            .returning(move |_, _, wanted| {
                log.lock().unwrap().push(wanted);
                let found = (wanted == version(1)).then(super::graph);
                Box::pin(async move { Ok(found) })
            });

        let (found, missing) = former_graphs(
            &mut surveys,
            &at_version_three(),
            vec![version(2), version(1), version(3), version(2), version(1)],
        )
        .await
        .unwrap();

        assert_eq!(*asked.lock().unwrap(), vec![version(2), version(1)]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].version(), version(1));
        assert_eq!(missing, vec![version(2)]);
    }

    #[tokio::test]
    async fn a_listing_with_a_missing_version_row_is_refused() {
        let mut surveys = MockSurveyRepository::new();
        surveys
            .expect_has_graph_version()
            .times(2)
            .returning(|_, _, wanted| Box::pin(async move { Ok(wanted == version(1)) }));

        let refused = require_graphs(
            &mut surveys,
            &at_version_three(),
            vec![version(3), version(1), version(1), version(2)],
        )
        .await;

        assert_eq!(
            refused.err().map(|error| error.to_string()),
            Some(common::CoreError::from(SurveyError::GraphVersionNotFound).to_string())
        );
    }
}
