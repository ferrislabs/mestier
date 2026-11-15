use std::num::NonZeroU32;

use chrono::{DateTime, Duration, Utc};
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    Answer, Answers, Campaign, CampaignId, CampaignStatus, MemberId, Metadata, OrganizationId,
    PublicSlug, QuestionId, QuestionKind, ResponseId, ResponseRecord, ResumeToken, Survey,
    SurveyBranch, SurveyEdge, SurveyEndNode, SurveyError, SurveyGraph, SurveyId, SurveyNode,
    SurveyQuestionNode, SurveyRecord, SurveyResponse, SurveyWindow, Visibility,
    application::test_support::{dev_pool, now_storable},
    domain::survey::ports::{CampaignRepository, ResponseRepository, SurveyRepository},
    infrastructure::{
        postgres::{SharedTx, with_tx},
        survey::postgres::{PgCampaignRepository, PgResponseRepository, PgSurveyRepository},
    },
};

pub(crate) struct Tenants {
    pub(crate) organization: OrganizationId,
    pub(crate) other: OrganizationId,
    pub(crate) member: MemberId,
}

fn qid(n: u128) -> QuestionId {
    QuestionId(Uuid::from_u128(n))
}

fn graph(identifier: bool) -> SurveyGraph {
    SurveyGraph::new(
        vec![
            SurveyNode::Question(SurveyQuestionNode {
                id: qid(10),
                kind: QuestionKind::Text,
                label: "Nom".to_string(),
                required: true,
                options: Vec::new(),
                max_photos: None,
                identifier,
            }),
            SurveyNode::End(SurveyEndNode {
                id: qid(11),
                message: Some("Merci".to_string()),
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

fn metadata(entries: &[(&str, &str)]) -> Metadata {
    Metadata::new(
        entries
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
    )
    .unwrap()
}

fn slug() -> PublicSlug {
    PublicSlug::new(generate_uuid_v7().simple().to_string()).unwrap()
}

fn token() -> ResumeToken {
    ResumeToken::new(generate_uuid_v7().simple().to_string()).unwrap()
}

fn answers(name: &str) -> Answers {
    Answers::from([(qid(10), Answer::Text(name.to_string()))])
}

fn draft(
    organization: OrganizationId,
    visibility: Visibility,
    campaign: Option<CampaignId>,
    identifier: bool,
) -> Survey<crate::Draft> {
    let mut survey = Survey::new(
        SurveyId(generate_uuid_v7()),
        organization,
        "Satisfaction".to_string(),
        visibility,
    )
    .unwrap();
    survey.edit_description(Some("Votre avis".to_string()));
    survey.edit_graph(Some(graph(identifier)));
    survey.edit_campaigns(campaign.into_iter().collect());
    survey.edit_response_cap(NonZeroU32::new(30));
    survey.edit_metadata(metadata(&[("theme", "survey")]));
    survey
}

fn opened(
    organization: OrganizationId,
    visibility: Visibility,
    identifier: bool,
    at: DateTime<Utc>,
) -> Survey<crate::Opened> {
    let public_slug = (visibility == Visibility::Public).then(slug);
    draft(organization, visibility, None, identifier)
        .open(at, public_slug)
        .unwrap()
}

fn campaign(
    organization: OrganizationId,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Campaign {
    Campaign::new(
        CampaignId(generate_uuid_v7()),
        organization,
        "Printemps".to_string(),
        SurveyWindow::new(starts_at, ends_at).unwrap(),
        metadata(&[("season", "spring")]),
        now,
    )
    .unwrap()
}

fn already_complete<T: std::fmt::Debug>(result: Result<T, CoreError>) {
    let expected = SurveyError::ResponseAlreadyCompleted.code();
    assert!(
        matches!(&result, Err(CoreError::Refused(refusal)) if refusal.reason == expected),
        "expected the response already complete error, got {result:?}"
    );
}

fn already_answered<T: std::fmt::Debug>(result: Result<T, CoreError>) {
    let expected = SurveyError::RespondentAlreadyAnswered.code();
    assert!(
        matches!(&result, Err(CoreError::Refused(refusal)) if refusal.reason == expected),
        "expected the respondent already answered error, got {result:?}"
    );
}

fn state_changed<T: std::fmt::Debug>(result: Result<T, CoreError>) {
    let expected = SurveyError::SurveyStateChanged.code();
    assert!(
        matches!(&result, Err(CoreError::Refused(refusal)) if refusal.reason == expected),
        "expected the survey state changed error, got {result:?}"
    );
}

pub(crate) async fn survey_contract<S: SurveyRepository>(
    surveys: &mut S,
    tenants: &Tenants,
) -> Result<(), CoreError> {
    let org = tenants.organization;
    let now = now_storable();

    let empty = SurveyRecord::from(Survey::new(
        SurveyId(generate_uuid_v7()),
        org,
        "Vide".to_string(),
        Visibility::Members,
    )?);
    assert_eq!(surveys.insert(&empty).await?, empty);
    assert_eq!(surveys.find(org, empty.id()).await?, Some(empty.clone()));

    let members = draft(org, Visibility::Members, None, false);
    let stored = SurveyRecord::from(members.clone());
    assert_eq!(surveys.insert(&stored).await?, stored);
    assert_eq!(surveys.find(org, stored.id()).await?, Some(stored.clone()));

    let stale_draft = stored.clone();
    let members = members.open(now, None)?;
    let stored = SurveyRecord::from(members.clone());
    assert_eq!(surveys.update(&stored).await?, stored);
    assert_eq!(surveys.find(org, stored.id()).await?, Some(stored.clone()));
    assert_eq!(
        surveys.find_for_update(org, stored.id()).await?,
        Some(stored.clone())
    );
    assert_eq!(
        surveys.find_for_update(tenants.other, stored.id()).await?,
        None
    );
    state_changed(surveys.update(&stale_draft).await);
    assert_eq!(surveys.find(org, stored.id()).await?, Some(stored.clone()));

    let stale_opened = stored.clone();
    let closed = SurveyRecord::from(members.close(now + Duration::hours(1))?);
    assert_eq!(surveys.update(&closed).await?, closed);
    assert_eq!(surveys.find(org, closed.id()).await?, Some(closed.clone()));
    state_changed(surveys.update(&stale_opened).await);
    state_changed(surveys.update(&stale_draft).await);
    assert_eq!(surveys.find(org, closed.id()).await?, Some(closed.clone()));

    let public_slug = slug();
    let public = SurveyRecord::from(
        draft(org, Visibility::Public, None, true).open(now, Some(public_slug.clone()))?,
    );
    assert_eq!(surveys.insert(&public).await?, public);
    assert_eq!(
        surveys.find_by_public_slug(&public_slug).await?,
        Some(public.clone())
    );
    assert_eq!(surveys.find_by_public_slug(&slug()).await?, None);

    assert_eq!(surveys.find(tenants.other, public.id()).await?, None);
    assert_eq!(surveys.list(tenants.other, 10, 0).await?, (vec![], 0));
    let (listed, total) = surveys.list(org, 10, 0).await?;
    assert_eq!(total, 3);
    assert_eq!(listed.len(), 3);
    assert!(listed.contains(&empty) && listed.contains(&closed) && listed.contains(&public));
    assert_eq!(surveys.list(org, 1, 1).await?.0.len(), 1);

    assert!(!surveys.has_responses(org, closed.id()).await?);
    surveys.delete(org, empty.id()).await?;
    assert_eq!(surveys.find(org, empty.id()).await?, None);
    assert!(matches!(
        surveys.delete(tenants.other, public.id()).await,
        Err(CoreError::NotFound)
    ));
    assert_eq!(surveys.find(org, public.id()).await?, Some(public));
    Ok(())
}

pub(crate) async fn campaign_contract<C: CampaignRepository, S: SurveyRepository>(
    campaigns: &mut C,
    surveys: &mut S,
    tenants: &Tenants,
) -> Result<(), CoreError> {
    let org = tenants.organization;
    let now = now_storable();

    let mut running = campaign(
        org,
        Some(now - Duration::days(1)),
        Some(now + Duration::days(1)),
        now,
    );
    assert_eq!(running.status(), CampaignStatus::Running);
    assert_eq!(campaigns.insert(&running).await?, running);
    assert_eq!(
        campaigns.find(org, running.id()).await?,
        Some(running.clone())
    );

    running.rename("Automne".to_string())?;
    running.edit_metadata(metadata(&[("season", "autumn"), ("year", "2026")]));
    running.reschedule(SurveyWindow::new(None, Some(now + Duration::days(2)))?, now);
    assert_eq!(campaigns.update(&running).await?, running);
    assert_eq!(
        campaigns.find(org, running.id()).await?,
        Some(running.clone())
    );

    assert_eq!(
        campaigns.find_for_update(org, running.id()).await?,
        Some(running.clone())
    );
    assert_eq!(
        campaigns
            .find_for_update(tenants.other, running.id())
            .await?,
        None
    );
    assert_eq!(campaigns.find(tenants.other, running.id()).await?, None);
    assert_eq!(campaigns.list(tenants.other, 10, 0).await?, (vec![], 0));
    assert_eq!(
        campaigns.list(org, 10, 0).await?,
        (vec![running.clone()], 1)
    );

    let attached = SurveyRecord::from(draft(org, Visibility::Members, Some(running.id()), false));
    surveys.insert(&attached).await?;
    assert_eq!(
        surveys.list_by_campaign(org, running.id()).await?,
        vec![attached]
    );
    assert_eq!(
        surveys
            .list_by_campaign(tenants.other, running.id())
            .await?,
        vec![]
    );

    let later = now + Duration::hours(2);
    let starting = campaign(org, Some(now + Duration::hours(1)), None, now);
    let ending = campaign(org, None, Some(now + Duration::hours(1)), now);
    let skipping = campaign(
        org,
        Some(now + Duration::minutes(10)),
        Some(now + Duration::minutes(20)),
        now,
    );
    let waiting = campaign(org, Some(now + Duration::days(1)), None, now);
    let ended = campaign(org, None, Some(now - Duration::hours(1)), now);
    assert_eq!(ended.status(), CampaignStatus::Ended);
    for stored in [&starting, &ending, &skipping, &waiting, &ended] {
        campaigns.insert(stored).await?;
    }
    let mine = [
        running.id(),
        starting.id(),
        ending.id(),
        skipping.id(),
        waiting.id(),
        ended.id(),
    ];
    let mut due: Vec<Campaign> = campaigns
        .list_due(later, 100_000)
        .await?
        .into_iter()
        .filter(|candidate| mine.contains(&candidate.id()))
        .collect();
    due.sort_by_key(|candidate| candidate.id().0);
    let mut expected = vec![starting, ending, skipping];
    expected.sort_by_key(|candidate| candidate.id().0);
    assert_eq!(due, expected);

    campaigns.delete(org, ended.id()).await?;
    assert_eq!(campaigns.find(org, ended.id()).await?, None);
    assert!(matches!(
        campaigns.delete(tenants.other, waiting.id()).await,
        Err(CoreError::NotFound)
    ));
    Ok(())
}

pub(crate) async fn response_contract<S: SurveyRepository, R: ResponseRepository>(
    surveys: &mut S,
    responses: &mut R,
    tenants: &Tenants,
) -> Result<(), CoreError> {
    let org = tenants.organization;
    let other = tenants.other;
    let now = now_storable();

    let members = opened(org, Visibility::Members, false, now);
    surveys.insert(&SurveyRecord::from(members.clone())).await?;
    let mut started = SurveyResponse::start(
        ResponseId(generate_uuid_v7()),
        &members,
        Some(tenants.member),
        None,
        now,
    )?;
    started.save(&members, answers("Claire"))?;
    let in_progress = ResponseRecord::from(started.clone());
    assert_eq!(responses.insert(&in_progress).await?, in_progress);
    assert_eq!(
        responses.find(org, in_progress.id()).await?,
        Some(in_progress.clone())
    );
    assert_eq!(
        responses
            .find_in_progress_by_member(org, members.id(), tenants.member)
            .await?,
        Some(in_progress.clone())
    );
    assert_eq!(
        responses
            .find_completed_by_member(org, members.id(), tenants.member)
            .await?,
        None
    );
    assert!(surveys.has_responses(org, members.id()).await?);

    assert_eq!(responses.find(other, in_progress.id()).await?, None);
    assert_eq!(responses.list(other, 10, 0).await?, (vec![], 0));
    assert_eq!(
        responses
            .list_for_survey(other, members.id(), 10, 0)
            .await?,
        (vec![], 0)
    );
    assert_eq!(
        responses
            .find_in_progress_by_member(other, members.id(), tenants.member)
            .await?,
        None
    );

    let completed = ResponseRecord::from(started.complete(&members, now + Duration::minutes(5))?);
    assert_eq!(responses.update(&completed).await?, completed);
    assert_eq!(
        responses.find(org, completed.id()).await?,
        Some(completed.clone())
    );
    assert_eq!(responses.count_completed(org, members.id()).await?, 1);
    assert_eq!(responses.count_completed(other, members.id()).await?, 0);
    assert_eq!(
        responses
            .find_completed_by_member(org, members.id(), tenants.member)
            .await?,
        Some(completed.clone())
    );
    assert_eq!(
        responses
            .find_in_progress_by_member(org, members.id(), tenants.member)
            .await?,
        None
    );
    already_complete(responses.update(&completed).await);
    assert_eq!(
        responses.list(org, 10, 0).await?,
        (vec![completed.clone()], 1)
    );
    assert_eq!(
        responses.list_for_survey(org, members.id(), 10, 0).await?,
        (vec![completed.clone()], 1)
    );

    let public = opened(org, Visibility::Public, true, now);
    surveys.insert(&SurveyRecord::from(public.clone())).await?;
    let resume = token();
    let mut anonymous = SurveyResponse::start(
        ResponseId(generate_uuid_v7()),
        &public,
        None,
        Some(resume.clone()),
        now,
    )?;
    anonymous.save(&public, answers("  Claire "))?;
    let stored = ResponseRecord::from(anonymous.clone());
    responses.insert(&stored).await?;
    assert_eq!(
        responses
            .find_by_resume_token(org, public.id(), &resume)
            .await?,
        Some(stored)
    );
    assert_eq!(
        responses
            .find_by_resume_token(other, public.id(), &resume)
            .await?,
        None
    );
    let anonymous = ResponseRecord::from(anonymous.complete(&public, now)?);
    responses.update(&anonymous).await?;
    assert_eq!(
        responses
            .find_completed_by_identifier(org, public.id(), "claire")
            .await?,
        Some(anonymous.clone())
    );
    assert_eq!(
        responses
            .find_completed_by_identifier(other, public.id(), "claire")
            .await?,
        None
    );

    assert!(matches!(
        responses.delete(other, anonymous.id()).await,
        Err(CoreError::NotFound)
    ));
    responses.delete(org, anonymous.id()).await?;
    assert_eq!(responses.find(org, anonymous.id()).await?, None);
    Ok(())
}

pub(crate) async fn completed_listing_contract<S: SurveyRepository, R: ResponseRepository>(
    surveys: &mut S,
    responses: &mut R,
    tenants: &Tenants,
) -> Result<(), CoreError> {
    let org = tenants.organization;
    let now = now_storable();
    let survey = opened(org, Visibility::Public, false, now);
    surveys.insert(&SurveyRecord::from(survey.clone())).await?;
    let mut completed = Vec::new();
    for minutes in [10, 30, 20] {
        let mut response = SurveyResponse::start(
            ResponseId(generate_uuid_v7()),
            &survey,
            None,
            Some(token()),
            now,
        )?;
        response.save(&survey, answers("Claire"))?;
        let done = response.complete(&survey, now + Duration::minutes(minutes))?;
        responses
            .insert(&ResponseRecord::from(done.clone()))
            .await?;
        completed.push(done);
    }
    for _ in 0..2 {
        let response = SurveyResponse::start(
            ResponseId(generate_uuid_v7()),
            &survey,
            None,
            Some(token()),
            now + Duration::minutes(40),
        )?;
        responses.insert(&ResponseRecord::from(response)).await?;
    }
    let newest_first = vec![
        completed[1].clone(),
        completed[2].clone(),
        completed[0].clone(),
    ];

    assert_eq!(
        responses
            .list_completed_for_survey(org, survey.id(), 10, 0)
            .await?,
        (newest_first.clone(), 3)
    );
    assert_eq!(
        responses
            .list_completed_for_survey(org, survey.id(), 1, 1)
            .await?,
        (vec![newest_first[1].clone()], 3)
    );
    assert_eq!(
        responses
            .list_completed_for_survey(tenants.other, survey.id(), 10, 0)
            .await?,
        (vec![], 0)
    );
    Ok(())
}

pub(crate) async fn member_uniqueness_contract<S: SurveyRepository, R: ResponseRepository>(
    surveys: &mut S,
    responses: &mut R,
    tenants: &Tenants,
) -> Result<(), CoreError> {
    let now = now_storable();
    let survey = opened(tenants.organization, Visibility::Members, false, now);
    surveys.insert(&SurveyRecord::from(survey.clone())).await?;
    let mut outcomes = Vec::new();
    for name in ["Claire", "Paul"] {
        let mut response = SurveyResponse::start(
            ResponseId(generate_uuid_v7()),
            &survey,
            Some(tenants.member),
            None,
            now,
        )?;
        response.save(&survey, answers(name))?;
        responses
            .insert(&ResponseRecord::from(response.clone()))
            .await?;
        let completed = ResponseRecord::from(response.complete(&survey, now)?);
        outcomes.push(responses.update(&completed).await);
    }
    assert!(outcomes[0].is_ok());
    already_answered(outcomes.remove(1));
    Ok(())
}

pub(crate) async fn identifier_uniqueness_contract<S: SurveyRepository, R: ResponseRepository>(
    surveys: &mut S,
    responses: &mut R,
    tenants: &Tenants,
) -> Result<(), CoreError> {
    let now = now_storable();
    let survey = opened(tenants.organization, Visibility::Public, true, now);
    surveys.insert(&SurveyRecord::from(survey.clone())).await?;
    let mut outcomes = Vec::new();
    for name in ["Claire", "claire "] {
        let mut response = SurveyResponse::start(
            ResponseId(generate_uuid_v7()),
            &survey,
            None,
            Some(token()),
            now,
        )?;
        response.save(&survey, answers(name))?;
        responses
            .insert(&ResponseRecord::from(response.clone()))
            .await?;
        let completed = ResponseRecord::from(response.complete(&survey, now)?);
        outcomes.push(responses.update(&completed).await);
    }
    assert!(outcomes[0].is_ok());
    already_answered(outcomes.remove(1));
    Ok(())
}

async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        format!("owner-{user_id}@example.com"),
        format!("owner-{user_id}"),
        "Owner",
        format!("sub-{user_id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#,
        org_id,
        "Test Org",
        format!("test-org-{org_id}"),
        user_id,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    OrganizationId(org_id)
}

async fn seed_member(tx: &SharedTx<'_>, organization_id: OrganizationId) -> MemberId {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, $3)"#,
        id,
        organization_id.0,
        "Claire",
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    MemberId(id)
}

async fn seed_tenants(tx: &SharedTx<'_>) -> Tenants {
    let organization = seed_organization(tx).await;
    let other = seed_organization(tx).await;
    let member = seed_member(tx, organization).await;
    Tenants {
        organization,
        other,
        member,
    }
}

async fn run<F>(work: F)
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<(), CoreError>,
{
    let pool: PgPool = dev_pool().await;
    let outcome = with_tx(&pool, async |tx| {
        work(tx).await?;
        Err::<(), _>(CoreError::Internal("rollback".into()))
    })
    .await;
    assert!(
        matches!(&outcome, Err(CoreError::Internal(message)) if message == "rollback"),
        "{outcome:?}"
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_honours_the_survey_contract() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        survey_contract(&mut PgSurveyRepository::new(&tx), &tenants).await
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_honours_the_campaign_contract() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        campaign_contract(
            &mut PgCampaignRepository::new(&tx),
            &mut PgSurveyRepository::new(&tx),
            &tenants,
        )
        .await
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_honours_the_response_contract() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        response_contract(
            &mut PgSurveyRepository::new(&tx),
            &mut PgResponseRepository::new(&tx),
            &tenants,
        )
        .await
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_lists_only_the_completed_responses_newest_first() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        completed_listing_contract(
            &mut PgSurveyRepository::new(&tx),
            &mut PgResponseRepository::new(&tx),
            &tenants,
        )
        .await
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_turns_a_second_member_completion_into_already_answered() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        member_uniqueness_contract(
            &mut PgSurveyRepository::new(&tx),
            &mut PgResponseRepository::new(&tx),
            &tenants,
        )
        .await
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_turns_a_reused_identifier_into_already_answered() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        identifier_uniqueness_contract(
            &mut PgSurveyRepository::new(&tx),
            &mut PgResponseRepository::new(&tx),
            &tenants,
        )
        .await
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_keeps_a_response_when_its_member_is_deleted() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        let mut surveys = PgSurveyRepository::new(&tx);
        let mut responses = PgResponseRepository::new(&tx);
        let now = now_storable();
        let survey = opened(tenants.organization, Visibility::Members, false, now);
        surveys.insert(&SurveyRecord::from(survey.clone())).await?;
        let mut response = SurveyResponse::start(
            ResponseId(generate_uuid_v7()),
            &survey,
            Some(tenants.member),
            None,
            now,
        )?;
        response.save(&survey, answers("Claire"))?;
        let completed = ResponseRecord::from(response.clone().complete(&survey, now)?);
        responses.insert(&completed).await?;
        {
            let mut guard = tx.lock().await;
            sqlx::query!(
                "DELETE FROM organization_members WHERE id = $1",
                tenants.member.0
            )
            .execute(&mut ***guard)
            .await
            .unwrap();
        }

        let Some(ResponseRecord::Completed(found)) =
            responses.find(tenants.organization, completed.id()).await?
        else {
            panic!("the completed response is gone");
        };
        assert_eq!(found.member_id(), None);
        assert_eq!(found.answers(), response.answers());
        assert_eq!(found.completed_at(), Some(now));
        assert_eq!(
            responses
                .find_completed_by_member(tenants.organization, survey.id(), tenants.member)
                .await?,
            None
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_keeps_the_campaigns_of_a_survey_in_attachment_order() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        let org = tenants.organization;
        let now = now_storable();
        let mut campaigns = PgCampaignRepository::new(&tx);
        let mut surveys = PgSurveyRepository::new(&tx);
        let first = campaign(org, None, None, now);
        let second = campaign(org, None, None, now);
        campaigns.insert(&first).await?;
        campaigns.insert(&second).await?;

        let mut survey = draft(org, Visibility::Members, None, false);
        survey.edit_campaigns(vec![second.id(), first.id()]);
        let stored = surveys.insert(&SurveyRecord::from(survey.clone())).await?;
        assert_eq!(stored.campaign_ids(), &[second.id(), first.id()]);

        survey.edit_campaigns(vec![first.id()]);
        let stored = surveys.update(&SurveyRecord::from(survey.clone())).await?;
        assert_eq!(stored.campaign_ids(), &[first.id()]);
        assert!(surveys.list_by_campaign(org, second.id()).await?.is_empty());

        survey.edit_campaigns(vec![first.id(), second.id()]);
        let opened = survey.open(now, None)?;
        let stored = surveys.update(&SurveyRecord::from(opened)).await?;
        assert_eq!(stored.campaign_ids(), &[first.id(), second.id()]);
        assert_eq!(
            surveys.find_for_update(org, stored.id()).await?,
            Some(stored.clone())
        );
        let other = surveys
            .insert(&SurveyRecord::from(draft(
                org,
                Visibility::Public,
                Some(second.id()),
                false,
            )))
            .await?;
        let mut held: Vec<SurveyId> = surveys
            .list_by_campaign(org, second.id())
            .await?
            .iter()
            .map(SurveyRecord::id)
            .collect();
        held.sort_by_key(|id| id.0);
        let mut expected = vec![stored.id(), other.id()];
        expected.sort_by_key(|id| id.0);
        assert_eq!(held, expected);
        assert!(
            surveys
                .list_by_campaign(tenants.other, second.id())
                .await?
                .is_empty()
        );
        assert!(campaigns.delete(org, second.id()).await.is_err());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn survey_postgres_stores_graph_versions_and_finds_completed_responses_per_version() {
    run(async |tx| {
        let tenants = seed_tenants(&tx).await;
        let org = tenants.organization;
        let now = now_storable();
        let mut surveys = PgSurveyRepository::new(&tx);
        let mut responses = PgResponseRepository::new(&tx);

        let mut members = opened(org, Visibility::Members, false, now);
        surveys.insert(&SurveyRecord::from(members.clone())).await?;
        assert!(
            !surveys
                .has_completed_responses(org, members.id(), 1)
                .await?
        );
        assert_eq!(
            surveys.find_graph_version(org, members.id(), 1).await?,
            None
        );

        let mut started = SurveyResponse::start(
            ResponseId(generate_uuid_v7()),
            &members,
            Some(tenants.member),
            None,
            now,
        )?;
        started.save(&members, answers("Claire"))?;
        responses
            .insert(&ResponseRecord::from(started.clone()))
            .await?;
        let completed = started.complete(&members, now)?;
        let saved = responses.update(&ResponseRecord::from(completed)).await?;
        assert_eq!(saved.graph_version(), Some(1));
        assert!(
            surveys
                .has_completed_responses(org, members.id(), 1)
                .await?
        );
        assert!(
            !surveys
                .has_completed_responses(org, members.id(), 2)
                .await?
        );
        assert!(
            !surveys
                .has_completed_responses(tenants.other, members.id(), 1)
                .await?
        );

        surveys
            .store_graph_version(org, members.id(), 1, members.definition())
            .await?;
        surveys
            .store_graph_version(org, members.id(), 1, &graph(true))
            .await?;
        assert_eq!(
            surveys.find_graph_version(org, members.id(), 1).await?,
            Some(graph(false))
        );
        assert_eq!(
            surveys
                .find_graph_version(tenants.other, members.id(), 1)
                .await?,
            None
        );

        let previous = members.revise_graph(graph(true));
        assert_eq!(previous.map(|stored| stored.version()), Some(1));
        let stored = surveys.update(&SurveyRecord::from(members.clone())).await?;
        assert_eq!(stored.graph_version(), Some(2));
        assert_eq!(stored.graph(), Some(&graph(true)));
        Ok(())
    })
    .await;
}
