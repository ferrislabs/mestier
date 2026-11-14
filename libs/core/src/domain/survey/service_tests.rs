use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, TimeZone, Utc};
use common::CoreError;
use events::testing::RecordingEmitter;
use serde_json::json;
use uuid::Uuid;

use super::ports::{MockCampaignRepository, MockResponseRepository, MockSurveyRepository};
use super::service::{
    CampaignPatch, CampaignService, ResponseLocator, ResponseService, SurveyPatch, SurveyService,
};
use super::{
    Answer, Answers, Campaign, CampaignId, CampaignStatus, Completed, Draft, Edge, EndNode, Graph,
    InProgress, Metadata, Node, Opened, PublicSlug, QuestionId, QuestionKind, QuestionNode,
    Response, ResponseId, ResponseRecord, ResumeToken, Survey, SurveyError, SurveyId, SurveyRecord,
    Visibility, Window,
};
use crate::{MemberId, OrganizationId};

type Surveys<'a> =
    SurveyService<MockSurveyRepository, MockCampaignRepository, &'a RecordingEmitter>;
type Campaigns<'a> =
    CampaignService<MockCampaignRepository, MockSurveyRepository, &'a RecordingEmitter>;
type Responses<'a> = ResponseService<
    MockResponseRepository,
    MockSurveyRepository,
    MockCampaignRepository,
    &'a RecordingEmitter,
>;

fn at(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, day, 12, 0, 0).unwrap()
}

fn org() -> OrganizationId {
    OrganizationId(Uuid::from_u128(900))
}

fn survey_id() -> SurveyId {
    SurveyId(Uuid::from_u128(1))
}

fn campaign_id() -> CampaignId {
    CampaignId(Uuid::from_u128(2))
}

fn member() -> MemberId {
    MemberId(Uuid::from_u128(3))
}

fn qid(n: u128) -> QuestionId {
    QuestionId(Uuid::from_u128(n))
}

fn graph(identifier: bool) -> Graph {
    Graph::new(
        vec![
            Node::Question(QuestionNode {
                id: qid(10),
                kind: QuestionKind::Text,
                label: "Nom".to_string(),
                required: true,
                options: Vec::new(),
                max_photos: None,
                identifier,
            }),
            Node::End(EndNode {
                id: qid(11),
                message: None,
            }),
        ],
        vec![Edge {
            from: qid(10),
            to: qid(11),
            branch: super::Branch::Next,
        }],
        qid(10),
    )
    .unwrap()
}

fn metadata(entries: &[(&str, &str)]) -> Metadata {
    Metadata::new(
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
    .unwrap()
}

struct Shape {
    visibility: Visibility,
    campaign: Option<CampaignId>,
    allow_multiple: bool,
    cap: Option<u32>,
    identifier: bool,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            visibility: Visibility::Members,
            campaign: None,
            allow_multiple: false,
            cap: None,
            identifier: false,
        }
    }
}

fn draft(shape: &Shape) -> Survey<Draft> {
    let mut survey = Survey::new(
        survey_id(),
        org(),
        "Satisfaction".to_string(),
        shape.visibility,
    )
    .unwrap();
    survey.edit_graph(Some(graph(shape.identifier)));
    survey.edit_campaign(shape.campaign);
    survey.edit_allow_multiple_responses(shape.allow_multiple);
    survey.edit_response_cap(shape.cap.and_then(NonZeroU32::new));
    survey.edit_metadata(metadata(&[("theme", "survey")]));
    survey
}

fn opened(shape: &Shape) -> Survey<Opened> {
    let slug = (shape.visibility == Visibility::Public)
        .then(|| PublicSlug::new("abcdefgh".to_string()).unwrap());
    draft(shape).open(at(1), slug).unwrap()
}

fn campaign(window: Window, now: DateTime<Utc>) -> Campaign {
    Campaign::new(
        campaign_id(),
        org(),
        "Printemps".to_string(),
        window,
        metadata(&[("season", "spring"), ("theme", "campaign")]),
        now,
    )
    .unwrap()
}

fn token() -> ResumeToken {
    ResumeToken::new("resume-token".to_string()).unwrap()
}

fn answers(name: &str) -> Answers {
    Answers::from([(qid(10), Answer::Text(name.to_string()))])
}

fn in_progress(survey: &Survey<Opened>, n: u128, name: &str) -> Response<InProgress> {
    let (member_id, resume_token) = match survey.visibility() {
        Visibility::Members => (Some(member()), None),
        Visibility::Public => (None, Some(token())),
    };
    let mut response = Response::start(
        ResponseId(Uuid::from_u128(n)),
        survey,
        member_id,
        resume_token,
        at(2),
    )
    .unwrap();
    response.save(survey, answers(name)).unwrap();
    response
}

fn completed(survey: &Survey<Opened>, n: u128, name: &str) -> Response<Completed> {
    in_progress(survey, n, name)
        .complete(survey, at(2))
        .unwrap()
}

fn surveys_holding(record: SurveyRecord) -> MockSurveyRepository {
    let mut surveys = MockSurveyRepository::new();
    let locked = record.clone();
    surveys.expect_find().returning(move |_, _| {
        let found = record.clone();
        Box::pin(async move { Ok(Some(found)) })
    });
    surveys.expect_find_for_update().returning(move |_, _| {
        let found = locked.clone();
        Box::pin(async move { Ok(Some(found)) })
    });
    surveys.expect_update().returning(|survey| {
        let saved = survey.clone();
        Box::pin(async move { Ok(saved) })
    });
    surveys.expect_insert().returning(|survey| {
        let saved = survey.clone();
        Box::pin(async move { Ok(saved) })
    });
    surveys
}

fn surveys_stored(stored: &Arc<Mutex<SurveyRecord>>) -> MockSurveyRepository {
    let mut surveys = MockSurveyRepository::new();
    let read = Arc::clone(stored);
    surveys
        .expect_find_for_update()
        .withf(|organization_id, id| *organization_id == org() && *id == survey_id())
        .returning(move |_, _| {
            let found = read.lock().unwrap().clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    let write = Arc::clone(stored);
    surveys.expect_update().returning(move |survey| {
        *write.lock().unwrap() = survey.clone();
        let saved = survey.clone();
        Box::pin(async move { Ok(saved) })
    });
    surveys
}

fn campaigns_holding(campaign: Option<Campaign>) -> MockCampaignRepository {
    let mut campaigns = MockCampaignRepository::new();
    campaigns.expect_find().returning(move |_, _| {
        let found = campaign.clone();
        Box::pin(async move { Ok(found) })
    });
    campaigns
}

fn responses_holding(record: ResponseRecord) -> MockResponseRepository {
    let mut responses = MockResponseRepository::new();
    let by_id = record.clone();
    responses.expect_find().returning(move |_, _| {
        let found = by_id.clone();
        Box::pin(async move { Ok(Some(found)) })
    });
    responses
        .expect_find_by_resume_token()
        .returning(move |_, _, _| {
            let found = record.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    responses.expect_update().returning(|response| {
        let saved = response.clone();
        Box::pin(async move { Ok(saved) })
    });
    responses
}

fn with_completed_count(responses: &mut MockResponseRepository, count: u64) {
    responses
        .expect_count_completed()
        .returning(move |_, _| Box::pin(async move { Ok(count) }));
}

fn refused<T: std::fmt::Debug>(result: Result<T, CoreError>, error: SurveyError) {
    assert_eq!(
        result.unwrap_err().to_string(),
        CoreError::from(error).to_string()
    );
}

fn member_locator() -> ResponseLocator {
    ResponseLocator::Member {
        id: ResponseId(Uuid::from_u128(50)),
        member_id: member(),
    }
}

#[tokio::test]
async fn opening_a_survey_emits_survey_opened_with_campaign_metadata() {
    let shape = Shape {
        campaign: Some(campaign_id()),
        ..Shape::default()
    };
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        surveys_holding(draft(&shape).into()),
        campaigns_holding(Some(campaign(Window::new(None, None).unwrap(), at(1)))),
        &emitter,
    );

    let survey = service.open(org(), survey_id(), at(1), None).await.unwrap();

    assert_eq!(survey.opened_at(), Some(at(1)));
    assert_eq!(emitter.names(), vec!["survey.opened"]);
    let event = emitter.only("survey.opened");
    assert_eq!(event.payload["campaign_id"], json!(campaign_id().0));
    assert_eq!(
        event.payload["metadata"],
        json!({"season": "spring", "theme": "survey"})
    );
}

#[tokio::test]
async fn a_draft_update_after_opening_is_refused_under_the_lock() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(draft(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut opening: Surveys<'_> =
        SurveyService::new(surveys_stored(&stored), campaigns_holding(None), &emitter);
    let mut editing: Surveys<'_> =
        SurveyService::new(surveys_stored(&stored), campaigns_holding(None), &emitter);

    opening.open(org(), survey_id(), at(1), None).await.unwrap();
    let stale = editing
        .update_draft(
            org(),
            survey_id(),
            SurveyPatch {
                title: Some("Trop tard".to_string()),
                ..SurveyPatch::default()
            },
        )
        .await;

    refused(stale, SurveyError::SurveyNotDraft);
    assert!(matches!(*stored.lock().unwrap(), SurveyRecord::Opened(_)));
}

#[tokio::test]
async fn opening_twice_emits_survey_opened_once() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(draft(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut first: Surveys<'_> =
        SurveyService::new(surveys_stored(&stored), campaigns_holding(None), &emitter);
    let mut second: Surveys<'_> =
        SurveyService::new(surveys_stored(&stored), campaigns_holding(None), &emitter);

    first.open(org(), survey_id(), at(1), None).await.unwrap();
    let again = second.open(org(), survey_id(), at(2), None).await;

    refused(again, SurveyError::SurveyNotDraft);
    assert_eq!(emitter.names(), vec!["survey.opened"]);
}

#[tokio::test]
async fn every_survey_transition_loads_the_survey_locked() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(draft(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut surveys = surveys_stored(&stored);
    surveys.expect_insert().returning(|survey| {
        let saved = survey.clone();
        Box::pin(async move { Ok(saved) })
    });
    surveys
        .expect_has_responses()
        .returning(|_, _| Box::pin(async { Ok(false) }));
    surveys
        .expect_delete()
        .returning(|_, _| Box::pin(async { Ok(()) }));
    let mut service: Surveys<'_> = SurveyService::new(surveys, campaigns_holding(None), &emitter);

    service
        .update_draft(org(), survey_id(), SurveyPatch::default())
        .await
        .unwrap();
    service
        .attach_campaign(org(), survey_id(), None)
        .await
        .unwrap();
    service
        .duplicate(org(), survey_id(), SurveyId(Uuid::from_u128(78)))
        .await
        .unwrap();
    service.open(org(), survey_id(), at(1), None).await.unwrap();
    service.close(org(), survey_id(), at(2)).await.unwrap();
    service.delete(org(), survey_id()).await.unwrap();
}

#[tokio::test]
async fn saving_emits_response_submitted() {
    let survey = opened(&Shape::default());
    let response = Response::start(
        ResponseId(Uuid::from_u128(50)),
        &survey,
        Some(member()),
        None,
        at(2),
    )
    .unwrap();
    let emitter = RecordingEmitter::new();
    let mut service: Responses<'_> = ResponseService::new(
        responses_holding(response.into()),
        surveys_holding(survey.into()),
        campaigns_holding(None),
        &emitter,
    );

    let saved = service
        .save(org(), &member_locator(), answers("Jane"), at(3))
        .await
        .unwrap();

    assert_eq!(saved.answers(), &answers("Jane"));
    assert_eq!(emitter.names(), vec!["survey.response_submitted"]);
    let event = emitter.only("survey.response_submitted");
    assert_eq!(event.payload["answered_questions"], 1);
}

#[tokio::test]
async fn completing_emits_response_completed_once() {
    let shape = Shape::default();
    let survey = opened(&shape);
    let emitter = RecordingEmitter::new();
    let mut responses = responses_holding(in_progress(&survey, 50, "Jane").into());
    with_completed_count(&mut responses, 0);
    responses
        .expect_find_completed_by_member()
        .returning(|_, _, _| Box::pin(async { Ok(None) }));
    let mut service: Responses<'_> = ResponseService::new(
        responses,
        surveys_holding(survey.clone().into()),
        campaigns_holding(None),
        &emitter,
    );

    let done = service
        .complete(org(), &member_locator(), at(3))
        .await
        .unwrap();

    let mut again: Responses<'_> = ResponseService::new(
        responses_holding(done.into()),
        surveys_holding(survey.into()),
        campaigns_holding(None),
        &emitter,
    );
    refused(
        again.complete(org(), &member_locator(), at(4)).await,
        SurveyError::ResponseAlreadyCompleted,
    );
    assert_eq!(emitter.names(), vec!["survey.response_completed"]);
    let event = emitter.only("survey.response_completed");
    assert_eq!(event.payload["answers"][0]["value"], "Jane");
}

#[tokio::test]
async fn a_member_cannot_complete_twice_when_multiple_responses_are_off() {
    let survey = opened(&Shape::default());
    let earlier = completed(&survey, 40, "Jane");
    let emitter = RecordingEmitter::new();
    let mut responses = MockResponseRepository::new();
    let current: ResponseRecord = in_progress(&survey, 50, "Jane").into();
    responses.expect_find().returning(move |_, _| {
        let found = current.clone();
        Box::pin(async move { Ok(Some(found)) })
    });
    with_completed_count(&mut responses, 1);
    responses
        .expect_find_completed_by_member()
        .returning(move |_, _, _| {
            let found = earlier.clone();
            Box::pin(async move { Ok(Some(found.into())) })
        });
    let mut service: Responses<'_> = ResponseService::new(
        responses,
        surveys_holding(survey.into()),
        campaigns_holding(None),
        &emitter,
    );

    refused(
        service.complete(org(), &member_locator(), at(3)).await,
        SurveyError::RespondentAlreadyAnswered,
    );
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn a_member_can_complete_twice_when_multiple_responses_are_on() {
    let survey = opened(&Shape {
        allow_multiple: true,
        ..Shape::default()
    });
    let emitter = RecordingEmitter::new();
    let mut responses = responses_holding(in_progress(&survey, 50, "Jane").into());
    with_completed_count(&mut responses, 1);
    let mut service: Responses<'_> = ResponseService::new(
        responses,
        surveys_holding(survey.into()),
        campaigns_holding(None),
        &emitter,
    );

    let done = service
        .complete(org(), &member_locator(), at(3))
        .await
        .unwrap();

    assert!(!done.unique_respondent());
    assert_eq!(emitter.names(), vec!["survey.response_completed"]);
}

#[tokio::test]
async fn a_public_identifier_already_used_is_refused() {
    let survey = opened(&Shape {
        visibility: Visibility::Public,
        identifier: true,
        ..Shape::default()
    });
    let earlier = completed(&survey, 40, "Jane");
    let emitter = RecordingEmitter::new();
    let mut responses = MockResponseRepository::new();
    let current: ResponseRecord = in_progress(&survey, 50, " JANE ").into();
    responses
        .expect_find_by_resume_token()
        .returning(move |_, _, _| {
            let found = current.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    with_completed_count(&mut responses, 1);
    responses
        .expect_find_completed_by_identifier()
        .withf(|_, _, value| value == "jane")
        .returning(move |_, _, _| {
            let found = earlier.clone();
            Box::pin(async move { Ok(Some(found.into())) })
        });
    let mut service: Responses<'_> = ResponseService::new(
        responses,
        surveys_holding(survey.into()),
        campaigns_holding(None),
        &emitter,
    );
    let locator = ResponseLocator::Public {
        survey_id: survey_id(),
        token: token(),
    };

    refused(
        service.complete(org(), &locator, at(3)).await,
        SurveyError::RespondentAlreadyAnswered,
    );
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn a_response_after_the_window_is_refused() {
    let shape = Shape {
        campaign: Some(campaign_id()),
        ..Shape::default()
    };
    let survey = opened(&shape);
    let window = Window::new(Some(at(2)), Some(at(5))).unwrap();
    let emitter = RecordingEmitter::new();
    let mut service: Responses<'_> = ResponseService::new(
        responses_holding(in_progress(&survey, 50, "Jane").into()),
        surveys_holding(survey.into()),
        campaigns_holding(Some(campaign(window, at(2)))),
        &emitter,
    );

    refused(
        service
            .save(org(), &member_locator(), answers("Jo"), at(5))
            .await,
        SurveyError::CampaignEnded,
    );
    refused(
        service.complete(org(), &member_locator(), at(6)).await,
        SurveyError::CampaignEnded,
    );
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn a_response_before_the_window_is_refused() {
    let shape = Shape {
        campaign: Some(campaign_id()),
        ..Shape::default()
    };
    let window = Window::new(Some(at(2)), Some(at(5))).unwrap();
    let emitter = RecordingEmitter::new();
    let mut service: Responses<'_> = ResponseService::new(
        MockResponseRepository::new(),
        surveys_holding(opened(&shape).into()),
        campaigns_holding(Some(campaign(window, at(1)))),
        &emitter,
    );

    refused(
        service
            .start_member(
                org(),
                survey_id(),
                ResponseId(Uuid::from_u128(50)),
                member(),
                at(2) - chrono::Duration::seconds(1),
            )
            .await,
        SurveyError::CampaignNotStarted,
    );
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn the_response_cap_refuses_the_completion_beyond_it() {
    let shape = Shape {
        cap: Some(2),
        allow_multiple: true,
        ..Shape::default()
    };
    let survey = opened(&shape);
    let emitter = RecordingEmitter::new();
    let mut responses = responses_holding(in_progress(&survey, 50, "Jane").into());
    with_completed_count(&mut responses, 2);
    responses
        .expect_find_in_progress_by_member()
        .returning(|_, _, _| Box::pin(async { Ok(None) }));
    responses.expect_insert().returning(|response| {
        let saved = response.clone();
        Box::pin(async move { Ok(saved) })
    });
    let mut service: Responses<'_> = ResponseService::new(
        responses,
        surveys_holding(survey.into()),
        campaigns_holding(None),
        &emitter,
    );

    let started = service
        .start_member(
            org(),
            survey_id(),
            ResponseId(Uuid::from_u128(60)),
            member(),
            at(3),
        )
        .await
        .unwrap();
    assert_eq!(started.id(), ResponseId(Uuid::from_u128(60)));
    refused(
        service.complete(org(), &member_locator(), at(3)).await,
        SurveyError::ResponseCapReached,
    );
    assert!(emitter.names().is_empty());
}

fn surveys_locked(record: SurveyRecord) -> MockSurveyRepository {
    let mut surveys = MockSurveyRepository::new();
    surveys
        .expect_find_for_update()
        .withf(|organization_id, id| *organization_id == org() && *id == survey_id())
        .returning(move |_, _| {
            let found = record.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    surveys
}

fn counting_responses(record: ResponseRecord, count: &Arc<Mutex<u64>>) -> MockResponseRepository {
    let mut responses = MockResponseRepository::new();
    responses.expect_find().returning(move |_, _| {
        let found = record.clone();
        Box::pin(async move { Ok(Some(found)) })
    });
    let read = Arc::clone(count);
    responses.expect_count_completed().returning(move |_, _| {
        let counted = *read.lock().unwrap();
        Box::pin(async move { Ok(counted) })
    });
    let write = Arc::clone(count);
    responses.expect_update().returning(move |response| {
        *write.lock().unwrap() += 1;
        let saved = response.clone();
        Box::pin(async move { Ok(saved) })
    });
    responses
}

#[tokio::test]
async fn the_last_completion_under_the_cap_is_accepted_and_the_next_refused() {
    let survey = opened(&Shape {
        cap: Some(2),
        allow_multiple: true,
        ..Shape::default()
    });
    let count = Arc::new(Mutex::new(1));
    let emitter = RecordingEmitter::new();
    let mut last: Responses<'_> = ResponseService::new(
        counting_responses(in_progress(&survey, 50, "Jane").into(), &count),
        surveys_locked(survey.clone().into()),
        campaigns_holding(None),
        &emitter,
    );
    let mut beyond: Responses<'_> = ResponseService::new(
        counting_responses(in_progress(&survey, 51, "Jo").into(), &count),
        surveys_locked(survey.into()),
        campaigns_holding(None),
        &emitter,
    );

    last.complete(org(), &member_locator(), at(3))
        .await
        .unwrap();
    refused(
        beyond.complete(org(), &member_locator(), at(3)).await,
        SurveyError::ResponseCapReached,
    );
    assert_eq!(*count.lock().unwrap(), 2);
    assert_eq!(emitter.names(), vec!["survey.response_completed"]);
}

#[tokio::test]
async fn a_completion_after_the_close_is_refused() {
    let survey = opened(&Shape::default());
    let response = in_progress(&survey, 50, "Jane");
    let closed = survey.close(at(3)).unwrap();
    let emitter = RecordingEmitter::new();
    let mut service: Responses<'_> = ResponseService::new(
        responses_holding(response.into()),
        surveys_locked(closed.into()),
        campaigns_holding(None),
        &emitter,
    );

    refused(
        service.complete(org(), &member_locator(), at(4)).await,
        SurveyError::SurveyClosed,
    );
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn deleting_a_survey_with_responses_is_refused() {
    let closed = opened(&Shape::default()).close(at(3)).unwrap();
    let mut surveys = surveys_holding(closed.into());
    surveys
        .expect_has_responses()
        .returning(|_, _| Box::pin(async { Ok(true) }));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> =
        SurveyService::new(surveys, MockCampaignRepository::new(), &emitter);

    refused(
        service.delete(org(), survey_id()).await,
        SurveyError::SurveyHasResponses,
    );
}

fn locked(campaign: Option<Campaign>) -> MockCampaignRepository {
    let mut campaigns = MockCampaignRepository::new();
    campaigns
        .expect_find_for_update()
        .withf(|organization_id, id| *organization_id == org() && *id == campaign_id())
        .times(1)
        .returning(move |_, _| {
            let found = campaign.clone();
            Box::pin(async move { Ok(found) })
        });
    campaigns
}

#[tokio::test]
async fn advance_rereads_the_campaign_locked_and_emits_started_then_ended() {
    let scheduled = campaign(Window::new(Some(at(2)), Some(at(4))).unwrap(), at(1));
    let mut campaigns = locked(Some(scheduled));
    campaigns
        .expect_update()
        .withf(|campaign| campaign.status() == CampaignStatus::Ended)
        .times(1)
        .returning(|campaign| {
            let saved = campaign.clone();
            Box::pin(async move { Ok(saved) })
        });
    let emitter = RecordingEmitter::new();
    let mut service: Campaigns<'_> =
        CampaignService::new(campaigns, MockSurveyRepository::new(), &emitter);

    let steps = service.advance(org(), campaign_id(), at(5)).await.unwrap();

    assert_eq!(steps, vec![CampaignStatus::Running, CampaignStatus::Ended]);
    assert_eq!(
        emitter.names(),
        vec!["survey_campaign.started", "survey_campaign.ended"]
    );
}

#[tokio::test]
async fn advance_emits_ended_only_from_running() {
    let running = campaign(Window::new(Some(at(2)), Some(at(4))).unwrap(), at(3));
    assert_eq!(running.status(), CampaignStatus::Running);
    let mut campaigns = locked(Some(running));
    campaigns
        .expect_update()
        .withf(|campaign| campaign.status() == CampaignStatus::Ended)
        .times(1)
        .returning(|campaign| {
            let saved = campaign.clone();
            Box::pin(async move { Ok(saved) })
        });
    let emitter = RecordingEmitter::new();
    let mut service: Campaigns<'_> =
        CampaignService::new(campaigns, MockSurveyRepository::new(), &emitter);

    let steps = service.advance(org(), campaign_id(), at(5)).await.unwrap();

    assert_eq!(steps, vec![CampaignStatus::Ended]);
    assert_eq!(emitter.names(), vec!["survey_campaign.ended"]);
}

#[tokio::test]
async fn advance_leaves_an_already_advanced_campaign_untouched() {
    let running = campaign(Window::new(Some(at(2)), Some(at(4))).unwrap(), at(3));
    let emitter = RecordingEmitter::new();
    let mut service: Campaigns<'_> =
        CampaignService::new(locked(Some(running)), MockSurveyRepository::new(), &emitter);

    let steps = service.advance(org(), campaign_id(), at(3)).await.unwrap();

    assert!(steps.is_empty());
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn advance_ignores_a_campaign_deleted_since_it_was_listed() {
    let emitter = RecordingEmitter::new();
    let mut service: Campaigns<'_> =
        CampaignService::new(locked(None), MockSurveyRepository::new(), &emitter);

    let steps = service.advance(org(), campaign_id(), at(5)).await.unwrap();

    assert!(steps.is_empty());
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn duplicate_creates_a_draft_without_slug_or_dates() {
    let shape = Shape {
        visibility: Visibility::Public,
        campaign: Some(campaign_id()),
        allow_multiple: true,
        cap: Some(5),
        ..Shape::default()
    };
    let closed = opened(&shape).close(at(3)).unwrap();
    let source_graph = closed.graph().cloned();
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        surveys_holding(closed.into()),
        MockCampaignRepository::new(),
        &emitter,
    );
    let new_id = SurveyId(Uuid::from_u128(77));

    let copy = service.duplicate(org(), survey_id(), new_id).await.unwrap();

    assert_eq!(copy.id(), new_id);
    assert_eq!(copy.title(), "Satisfaction");
    assert_eq!(copy.visibility(), Visibility::Public);
    assert_eq!(copy.campaign_id(), Some(campaign_id()));
    assert!(copy.allow_multiple_responses());
    assert_eq!(copy.response_cap(), NonZeroU32::new(5));
    assert_eq!(copy.metadata(), &metadata(&[("theme", "survey")]));
    assert_eq!(copy.graph().cloned(), source_graph);
    assert_eq!(copy.opened_at(), None);
    assert_eq!(copy.closed_at(), None);
    let json = serde_json::to_value(&copy).unwrap();
    assert!(json["public_slug"].is_null());
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn editing_a_campaign_loads_it_locked() {
    let running = campaign(Window::new(Some(at(2)), Some(at(4))).unwrap(), at(3));
    let mut campaigns = locked(Some(running));
    campaigns
        .expect_update()
        .withf(|campaign| campaign.status() == CampaignStatus::Ended)
        .times(1)
        .returning(|campaign| {
            let saved = campaign.clone();
            Box::pin(async move { Ok(saved) })
        });
    let emitter = RecordingEmitter::new();
    let mut service: Campaigns<'_> =
        CampaignService::new(campaigns, MockSurveyRepository::new(), &emitter);

    let saved = service
        .update(
            org(),
            campaign_id(),
            CampaignPatch {
                name: Some("Automne".to_string()),
                ..CampaignPatch::default()
            },
            at(5),
        )
        .await
        .unwrap();

    assert_eq!(saved.name(), "Automne");
    assert_eq!(emitter.names(), vec!["survey_campaign.ended"]);
}

#[tokio::test]
async fn deleting_a_campaign_loads_it_locked() {
    let scheduled = campaign(Window::new(Some(at(2)), Some(at(4))).unwrap(), at(1));
    let mut campaigns = locked(Some(scheduled));
    campaigns
        .expect_delete()
        .times(1)
        .returning(|_, _| Box::pin(async { Ok(()) }));
    let mut surveys = MockSurveyRepository::new();
    surveys
        .expect_list_by_campaign()
        .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));
    let emitter = RecordingEmitter::new();
    let mut service: Campaigns<'_> = CampaignService::new(campaigns, surveys, &emitter);

    service.delete(org(), campaign_id()).await.unwrap();
}

fn keys(value: &serde_json::Value) -> Vec<String> {
    let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

#[test]
fn the_respondent_view_of_a_public_survey_hides_the_organization_and_the_settings() {
    let survey = opened(&Shape {
        visibility: Visibility::Public,
        campaign: Some(campaign_id()),
        allow_multiple: true,
        cap: Some(5),
        ..Shape::default()
    });

    let view = survey.respondent_view();
    let json = serde_json::to_value(&view).unwrap();

    assert_eq!(
        keys(&json),
        vec![
            "description",
            "graph",
            "id",
            "public_slug",
            "title",
            "visibility"
        ]
    );
    assert_eq!(json["public_slug"], "abcdefgh");
    assert_eq!(json["title"], "Satisfaction");
    assert_eq!(
        json["graph"],
        serde_json::to_value(survey.definition()).unwrap()
    );
    assert_eq!(view.id(), survey_id());
}

#[test]
fn the_respondent_view_of_a_members_survey_has_no_public_slug() {
    let survey = opened(&Shape::default());

    let json = serde_json::to_value(survey.respondent_view()).unwrap();

    assert_eq!(
        keys(&json),
        vec!["description", "graph", "id", "title", "visibility"]
    );
}
