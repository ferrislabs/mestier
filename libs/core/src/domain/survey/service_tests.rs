use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, TimeZone, Utc};
use common::CoreError;
use events::testing::RecordingEmitter;
use serde_json::json;
use uuid::Uuid;

use super::ports::{MockCampaignRepository, MockResponseRepository, MockSurveyRepository};
use super::service::{
    CampaignPatch, CampaignService, ResponseLocator, ResponseService, StartedResponse, SurveyPatch,
    SurveyService,
};
use super::{
    Answer, Answers, Campaign, CampaignId, CampaignStatus, Completed, Draft, Edge, EndNode, Graph,
    GraphVersion, InProgress, Metadata, Node, Opened, PublicSlug, QuestionId, QuestionKind,
    QuestionNode, Response, ResponseId, ResponseRecord, ResumeToken, Survey, SurveyError, SurveyId,
    SurveyRecord, Visibility, Window,
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
                block: None,
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
    survey.edit_campaigns(shape.campaign.into_iter().collect());
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
        .expect_store_graph_version()
        .returning(|_, _, _, _| Box::pin(async { Ok(()) }));
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
        .expect_has_graph_version()
        .returning(|_, _, _| Box::pin(async { Ok(false) }));
    let attach = Arc::clone(stored);
    surveys
        .expect_save_campaigns()
        .withf(|organization_id, id, _| *organization_id == org() && *id == survey_id())
        .returning(move |_, _, campaign_ids| {
            let mut stored = attach.lock().unwrap();
            let edited = match stored.clone() {
                SurveyRecord::Draft(mut survey) => {
                    survey.edit_campaigns(campaign_ids.to_vec());
                    SurveyRecord::Draft(survey)
                }
                SurveyRecord::Opened(mut survey) => {
                    survey.edit_campaigns(campaign_ids.to_vec());
                    SurveyRecord::Opened(survey)
                }
                SurveyRecord::Closed(mut survey) => {
                    survey.edit_campaigns(campaign_ids.to_vec());
                    SurveyRecord::Closed(survey)
                }
            };
            *stored = edited;
            Box::pin(async { Ok(()) })
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
    assert_eq!(event.payload["campaign_ids"], json!([campaign_id().0]));
    assert_eq!(
        event.payload["metadata"],
        json!({"season": "spring", "theme": "survey"})
    );
}

#[tokio::test]
async fn an_edit_after_opening_applies_to_the_opened_survey_under_the_lock() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(draft(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut opening: Surveys<'_> =
        SurveyService::new(surveys_stored(&stored), campaigns_holding(None), &emitter);
    let mut editing: Surveys<'_> =
        SurveyService::new(surveys_stored(&stored), campaigns_holding(None), &emitter);

    opening.open(org(), survey_id(), at(1), None).await.unwrap();
    editing
        .update(
            org(),
            survey_id(),
            SurveyPatch {
                title: Some("Après ouverture".to_string()),
                ..SurveyPatch::default()
            },
        )
        .await
        .unwrap();

    match &*stored.lock().unwrap() {
        SurveyRecord::Opened(survey) => assert_eq!(survey.title(), "Après ouverture"),
        other => panic!("expected an opened survey, got {other:?}"),
    }
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
        .update(org(), survey_id(), SurveyPatch::default())
        .await
        .unwrap();
    service
        .set_campaigns(org(), survey_id(), Vec::new())
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
    assert_eq!(
        started.into_response().id(),
        ResponseId(Uuid::from_u128(60))
    );
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
        .expect_store_graph_version()
        .returning(|_, _, _, _| Box::pin(async { Ok(()) }));
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
    assert_eq!(copy.campaign_ids(), &[campaign_id()]);
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
        .expect_list_summaries_by_campaign()
        .returning(|_, _, _, _| Box::pin(async { Ok((Vec::new(), 0)) }));
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

#[tokio::test]
async fn a_draft_survey_is_not_found_by_its_respondents() {
    let emitter = RecordingEmitter::new();
    let mut members: Responses<'_> = ResponseService::new(
        MockResponseRepository::new(),
        surveys_holding(draft(&Shape::default()).into()),
        campaigns_holding(None),
        &emitter,
    );
    let mut visitors: Responses<'_> = ResponseService::new(
        MockResponseRepository::new(),
        surveys_holding(
            draft(&Shape {
                visibility: Visibility::Public,
                ..Shape::default()
            })
            .into(),
        ),
        campaigns_holding(None),
        &emitter,
    );

    refused(
        members
            .start_member(
                org(),
                survey_id(),
                ResponseId(Uuid::from_u128(50)),
                member(),
                at(2),
            )
            .await,
        SurveyError::SurveyNotFound,
    );
    refused(
        visitors
            .start_public(
                org(),
                survey_id(),
                ResponseId(Uuid::from_u128(51)),
                token(),
                at(2),
            )
            .await,
        SurveyError::SurveyNotFound,
    );
    assert!(emitter.names().is_empty());
}

fn public() -> Shape {
    Shape {
        visibility: Visibility::Public,
        ..Shape::default()
    }
}

#[tokio::test]
async fn a_member_path_does_not_find_a_public_survey() {
    let emitter = RecordingEmitter::new();
    let members_survey = opened(&Shape::default());
    let mut service: Responses<'_> = ResponseService::new(
        responses_holding(in_progress(&members_survey, 50, "Jane").into()),
        surveys_holding(opened(&public()).into()),
        campaigns_holding(None),
        &emitter,
    );

    refused(
        service
            .start_member(
                org(),
                survey_id(),
                ResponseId(Uuid::from_u128(60)),
                member(),
                at(2),
            )
            .await,
        SurveyError::SurveyNotFound,
    );
    refused(
        service
            .save(org(), &member_locator(), answers("Jane"), at(2))
            .await,
        SurveyError::SurveyNotFound,
    );
    refused(
        service.complete(org(), &member_locator(), at(2)).await,
        SurveyError::SurveyNotFound,
    );
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn a_public_path_does_not_find_a_members_survey() {
    let emitter = RecordingEmitter::new();
    let public_survey = opened(&public());
    let mut service: Responses<'_> = ResponseService::new(
        responses_holding(in_progress(&public_survey, 50, "Jane").into()),
        surveys_holding(opened(&Shape::default()).into()),
        campaigns_holding(None),
        &emitter,
    );
    let locator = ResponseLocator::Public {
        survey_id: survey_id(),
        token: token(),
    };

    refused(
        service
            .start_public(
                org(),
                survey_id(),
                ResponseId(Uuid::from_u128(60)),
                token(),
                at(2),
            )
            .await,
        SurveyError::SurveyNotFound,
    );
    refused(
        service.save(org(), &locator, answers("Jane"), at(2)).await,
        SurveyError::SurveyNotFound,
    );
    refused(
        service.complete(org(), &locator, at(2)).await,
        SurveyError::SurveyNotFound,
    );
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn starting_tells_a_created_response_from_a_resumed_one() {
    let survey = opened(&Shape::default());
    let existing = in_progress(&survey, 50, "Jane");
    let emitter = RecordingEmitter::new();
    let mut fresh = MockResponseRepository::new();
    fresh
        .expect_find_in_progress_by_member()
        .returning(|_, _, _| Box::pin(async { Ok(None) }));
    fresh
        .expect_find_completed_by_member()
        .returning(|_, _, _| Box::pin(async { Ok(None) }));
    fresh.expect_insert().returning(|response| {
        let saved = response.clone();
        Box::pin(async move { Ok(saved) })
    });
    let mut resuming = MockResponseRepository::new();
    let found = ResponseRecord::from(existing.clone());
    resuming
        .expect_find_in_progress_by_member()
        .returning(move |_, _, _| {
            let found = found.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    let mut creating: Responses<'_> = ResponseService::new(
        fresh,
        surveys_holding(survey.clone().into()),
        campaigns_holding(None),
        &emitter,
    );
    let mut resumed: Responses<'_> = ResponseService::new(
        resuming,
        surveys_holding(survey.into()),
        campaigns_holding(None),
        &emitter,
    );

    let created = creating
        .start_member(
            org(),
            survey_id(),
            ResponseId(Uuid::from_u128(60)),
            member(),
            at(2),
        )
        .await
        .unwrap();
    let again = resumed
        .start_member(
            org(),
            survey_id(),
            ResponseId(Uuid::from_u128(61)),
            member(),
            at(2),
        )
        .await
        .unwrap();

    assert!(
        matches!(created, StartedResponse::Created(ref response) if response.id() == ResponseId(Uuid::from_u128(60)))
    );
    assert!(!created.is_resumed());
    assert!(again.is_resumed());
    assert_eq!(again.into_response(), existing);
}

fn other_graph() -> Graph {
    Graph::new(
        vec![
            Node::Question(QuestionNode {
                id: qid(20),
                kind: QuestionKind::Number,
                label: "Âge".to_string(),
                required: false,
                options: Vec::new(),
                max_photos: None,
                identifier: false,
                block: None,
            }),
            Node::End(EndNode {
                id: qid(21),
                message: None,
            }),
        ],
        vec![Edge {
            from: qid(20),
            to: qid(21),
            branch: super::Branch::Next,
        }],
        qid(20),
    )
    .unwrap()
}

fn graph_patch(graph: Option<Graph>) -> SurveyPatch {
    SurveyPatch {
        graph: Some(graph),
        ..SurveyPatch::default()
    }
}

fn versioning_surveys(stored: &Arc<Mutex<SurveyRecord>>, frozen: bool) -> MockSurveyRepository {
    let mut surveys = MockSurveyRepository::new();
    let read = Arc::clone(stored);
    surveys.expect_find_for_update().returning(move |_, _| {
        let found = read.lock().unwrap().clone();
        Box::pin(async move { Ok(Some(found)) })
    });
    let write = Arc::clone(stored);
    surveys.expect_update().returning(move |survey| {
        *write.lock().unwrap() = survey.clone();
        let saved = survey.clone();
        Box::pin(async move { Ok(saved) })
    });
    let current = Arc::clone(stored);
    surveys
        .expect_has_graph_version()
        .withf(move |organization_id, id, version| {
            *organization_id == org()
                && *id == survey_id()
                && Some(*version) == current.lock().unwrap().graph_version()
        })
        .returning(move |_, _, _| Box::pin(async move { Ok(frozen) }));
    surveys.expect_store_graph_version().never();
    surveys.expect_save_campaigns().never();
    surveys
}

fn version(number: u32) -> GraphVersion {
    GraphVersion::new(number).unwrap()
}

#[tokio::test]
async fn editing_the_graph_of_a_frozen_version_starts_a_new_version() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(opened(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        versioning_surveys(&stored, true),
        campaigns_holding(None),
        &emitter,
    );

    let edited = service
        .update(org(), survey_id(), graph_patch(Some(other_graph())))
        .await
        .unwrap();

    assert_eq!(edited.graph_version(), Some(version(2)));
    assert_eq!(edited.graph(), Some(&other_graph()));
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn editing_the_graph_of_an_unfrozen_version_overwrites_it_in_place() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(opened(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        versioning_surveys(&stored, false),
        campaigns_holding(None),
        &emitter,
    );

    service
        .update(org(), survey_id(), graph_patch(Some(other_graph())))
        .await
        .unwrap();
    let edited = service
        .update(org(), survey_id(), graph_patch(Some(graph(true))))
        .await
        .unwrap();

    assert_eq!(edited.graph_version(), Some(GraphVersion::FIRST));
    assert_eq!(edited.graph(), Some(&graph(true)));
}

#[tokio::test]
async fn editing_the_last_graph_version_is_refused() {
    let mut json = serde_json::to_value(opened(&Shape::default())).unwrap();
    json["graph_version"] = json!(u32::MAX);
    let last: Survey<Opened> = serde_json::from_value(json).unwrap();
    let stored = Arc::new(Mutex::new(SurveyRecord::from(last)));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        versioning_surveys(&stored, true),
        campaigns_holding(None),
        &emitter,
    );

    let edited = service
        .update(org(), survey_id(), graph_patch(Some(other_graph())))
        .await;

    refused(edited, SurveyError::GraphVersionExhausted);
}

#[tokio::test]
async fn a_closed_survey_is_edited_and_stays_closed() {
    let closed = opened(&Shape::default()).close(at(3)).unwrap();
    let stored = Arc::new(Mutex::new(SurveyRecord::from(closed)));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        versioning_surveys(&stored, true),
        campaigns_holding(None),
        &emitter,
    );

    let edited = service
        .update(
            org(),
            survey_id(),
            SurveyPatch {
                title: Some("Bilan".to_string()),
                graph: Some(Some(other_graph())),
                ..SurveyPatch::default()
            },
        )
        .await
        .unwrap();

    let SurveyRecord::Closed(edited) = edited else {
        panic!("the survey stays closed");
    };
    assert_eq!(edited.title(), "Bilan");
    assert_eq!(edited.graph_version(), Some(version(2)));
}

#[tokio::test]
async fn the_visibility_of_an_opened_survey_cannot_change() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(opened(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        versioning_surveys(&stored, false),
        campaigns_holding(None),
        &emitter,
    );

    let changed = service
        .update(
            org(),
            survey_id(),
            SurveyPatch {
                visibility: Some(Visibility::Public),
                ..SurveyPatch::default()
            },
        )
        .await;
    refused(changed, SurveyError::SurveyVisibilityFrozen);
    assert_eq!(
        SurveyError::SurveyVisibilityFrozen.code(),
        "survey_visibility_frozen"
    );

    service
        .update(
            org(),
            survey_id(),
            SurveyPatch {
                visibility: Some(Visibility::Members),
                ..SurveyPatch::default()
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn an_opened_survey_cannot_lose_its_graph() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(opened(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        versioning_surveys(&stored, false),
        campaigns_holding(None),
        &emitter,
    );

    let cleared = service.update(org(), survey_id(), graph_patch(None)).await;

    refused(cleared, SurveyError::SurveyWithoutGraph);
}

#[tokio::test]
async fn setting_the_campaigns_of_an_opened_survey_checks_each_one() {
    let stored = Arc::new(Mutex::new(SurveyRecord::from(opened(&Shape::default()))));
    let emitter = RecordingEmitter::new();
    let mut campaigns = MockCampaignRepository::new();
    campaigns.expect_find().returning(|_, id| {
        let found =
            (id == campaign_id()).then(|| campaign(Window::new(None, None).unwrap(), at(1)));
        Box::pin(async move { Ok(found) })
    });
    let mut service: Surveys<'_> = SurveyService::new(surveys_stored(&stored), campaigns, &emitter);

    let unknown = service
        .set_campaigns(
            org(),
            survey_id(),
            vec![campaign_id(), CampaignId(Uuid::from_u128(99))],
        )
        .await;
    refused(unknown, SurveyError::CampaignNotFound);

    let set = service
        .set_campaigns(org(), survey_id(), vec![campaign_id(), campaign_id()])
        .await
        .unwrap();
    assert!(matches!(set, SurveyRecord::Opened(_)));
    assert_eq!(set.campaign_ids(), &[campaign_id()]);
    assert_eq!(stored.lock().unwrap().campaign_ids(), &[campaign_id()]);
}

fn campaign_saving_surveys(
    record: SurveyRecord,
    saved: &Arc<Mutex<Vec<Vec<CampaignId>>>>,
) -> MockSurveyRepository {
    let mut surveys = MockSurveyRepository::new();
    surveys.expect_find().never();
    surveys.expect_update().never();
    surveys
        .expect_find_for_update()
        .withf(|organization_id, id| *organization_id == org() && *id == survey_id())
        .times(1)
        .returning(move |_, _| {
            let found = record.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    let write = Arc::clone(saved);
    surveys
        .expect_save_campaigns()
        .withf(|organization_id, id, _| *organization_id == org() && *id == survey_id())
        .returning(move |_, _, campaign_ids| {
            write.lock().unwrap().push(campaign_ids.to_vec());
            Box::pin(async { Ok(()) })
        });
    surveys
}

fn any_campaign() -> MockCampaignRepository {
    let mut campaigns = MockCampaignRepository::new();
    campaigns.expect_find().returning(|_, id| {
        let found = Campaign::new(
            id,
            org(),
            "Printemps".to_string(),
            Window::new(None, None).unwrap(),
            Metadata::default(),
            at(1),
        )
        .ok();
        Box::pin(async move { Ok(found) })
    });
    campaigns
}

#[tokio::test]
async fn setting_the_campaigns_saves_only_the_campaign_set_under_the_lock() {
    let saved = Arc::new(Mutex::new(Vec::new()));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        campaign_saving_surveys(draft(&Shape::default()).into(), &saved),
        any_campaign(),
        &emitter,
    );

    service
        .set_campaigns(org(), survey_id(), vec![campaign_id()])
        .await
        .unwrap();

    assert_eq!(*saved.lock().unwrap(), vec![vec![campaign_id()]]);
}

#[tokio::test]
async fn adding_a_campaign_appends_it_to_the_locked_survey() {
    let other = CampaignId(Uuid::from_u128(77));
    let attached = draft(&Shape {
        campaign: Some(campaign_id()),
        ..Shape::default()
    });
    let saved = Arc::new(Mutex::new(Vec::new()));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        campaign_saving_surveys(attached.into(), &saved),
        any_campaign(),
        &emitter,
    );

    let added = service
        .add_campaign(org(), survey_id(), other)
        .await
        .unwrap();

    assert_eq!(added.campaign_ids(), &[campaign_id(), other]);
    assert_eq!(*saved.lock().unwrap(), vec![vec![campaign_id(), other]]);
}

#[tokio::test]
async fn adding_a_campaign_already_attached_keeps_the_set() {
    let attached = draft(&Shape {
        campaign: Some(campaign_id()),
        ..Shape::default()
    });
    let saved = Arc::new(Mutex::new(Vec::new()));
    let emitter = RecordingEmitter::new();
    let mut service: Surveys<'_> = SurveyService::new(
        campaign_saving_surveys(attached.into(), &saved),
        any_campaign(),
        &emitter,
    );

    let added = service
        .add_campaign(org(), survey_id(), campaign_id())
        .await
        .unwrap();

    assert_eq!(added.campaign_ids(), &[campaign_id()]);
    assert_eq!(*saved.lock().unwrap(), vec![vec![campaign_id()]]);
}

#[tokio::test]
async fn completing_records_the_graph_version_of_the_survey() {
    let survey = opened(&Shape::default());
    let emitter = RecordingEmitter::new();
    let mut responses = responses_holding(in_progress(&survey, 50, "Jane").into());
    with_completed_count(&mut responses, 0);
    responses
        .expect_find_completed_by_member()
        .returning(|_, _, _| Box::pin(async { Ok(None) }));
    let mut surveys = MockSurveyRepository::new();
    let found = survey.clone();
    surveys.expect_find_for_update().returning(move |_, _| {
        let found = found.clone().into();
        Box::pin(async move { Ok(Some(found)) })
    });
    surveys
        .expect_store_graph_version()
        .withf(|organization_id, id, version, stored| {
            *organization_id == org()
                && *id == survey_id()
                && *version == GraphVersion::FIRST
                && *stored == graph(false)
        })
        .times(1)
        .returning(|_, _, _, _| Box::pin(async { Ok(()) }));
    let mut service: Responses<'_> =
        ResponseService::new(responses, surveys, campaigns_holding(None), &emitter);

    let done = service
        .complete(org(), &member_locator(), at(3))
        .await
        .unwrap();

    assert_eq!(done.graph_version(), GraphVersion::FIRST);
    assert_eq!(
        emitter.only("survey.response_completed").payload["graph_version"],
        json!(1)
    );
}
