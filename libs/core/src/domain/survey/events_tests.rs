use chrono::{DateTime, TimeZone, Utc};
use events::DomainEvent;
use serde_json::{Value, json};
use uuid::Uuid;

use super::events::{
    SurveyCampaignEnded, SurveyCampaignStarted, SurveyOpened, SurveyResponseCompleted,
    SurveyResponseSubmitted, descriptors, emitted_events,
};
use super::{
    Answer, Answers, Branch, Campaign, CampaignId, ConditionNode, Draft, Edge, EndNode, Graph,
    Metadata, Node, Opened, QuestionId, QuestionKind, QuestionNode, Response, ResponseId, Survey,
    SurveyError, SurveyId, Visibility, Window,
};
use crate::{MemberId, OrganizationId};

fn at(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, day, 12, 0, 0).unwrap()
}

fn qid(n: u128) -> QuestionId {
    QuestionId(Uuid::from_u128(n))
}

fn org() -> OrganizationId {
    OrganizationId(Uuid::from_u128(900))
}

fn question(n: u128, kind: QuestionKind, required: bool) -> Node {
    Node::Question(QuestionNode {
        id: qid(n),
        kind,
        label: format!("Question {n}"),
        required,
        options: if kind == QuestionKind::SingleChoice {
            vec!["Oui".to_string(), "Non".to_string()]
        } else {
            Vec::new()
        },
        max_photos: None,
        identifier: false,
    })
}

fn edge(from: u128, to: u128, branch: Branch) -> Edge {
    Edge {
        from: qid(from),
        to: qid(to),
        branch,
    }
}

fn graph() -> Graph {
    Graph::new(
        vec![
            question(1, QuestionKind::SingleChoice, true),
            Node::Condition(ConditionNode {
                id: qid(2),
                question: qid(1),
                option: "Oui".to_string(),
                checked: true,
            }),
            question(3, QuestionKind::Text, true),
            question(4, QuestionKind::Text, false),
            question(5, QuestionKind::Number, false),
            Node::End(EndNode {
                id: qid(6),
                message: None,
            }),
        ],
        vec![
            edge(1, 2, Branch::Next),
            edge(2, 3, Branch::Then),
            edge(2, 4, Branch::Else),
            edge(3, 5, Branch::Next),
            edge(4, 5, Branch::Next),
            edge(5, 6, Branch::Next),
        ],
        qid(1),
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

fn draft(campaign: Option<CampaignId>) -> Survey<Draft> {
    let mut survey = Survey::new(
        SurveyId(Uuid::from_u128(1)),
        org(),
        "Satisfaction".to_string(),
        Visibility::Members,
    )
    .unwrap();
    survey.edit_graph(Some(graph()));
    survey.edit_campaign(campaign);
    survey.edit_metadata(metadata(&[("theme", "survey")]));
    survey.edit_description(Some("Après chantier".to_string()));
    survey
}

fn opened(campaign: Option<CampaignId>) -> Survey<Opened> {
    draft(campaign).open(at(1), None).unwrap()
}

fn conflicting_campaign() -> Campaign {
    Campaign::new(
        CampaignId(Uuid::from_u128(2)),
        org(),
        "Printemps".to_string(),
        Window::new(Some(at(1)), None).unwrap(),
        metadata(&[("season", "spring"), ("theme", "campaign")]),
        at(1),
    )
    .unwrap()
}

fn foreign_campaign() -> Campaign {
    Campaign::new(
        CampaignId(Uuid::from_u128(7)),
        org(),
        "Autre".to_string(),
        Window::new(None, None).unwrap(),
        Metadata::default(),
        at(1),
    )
    .unwrap()
}

fn campaign() -> Campaign {
    Campaign::new(
        CampaignId(Uuid::from_u128(2)),
        org(),
        "Printemps".to_string(),
        Window::new(Some(at(1)), None).unwrap(),
        metadata(&[("season", "spring")]),
        at(1),
    )
    .unwrap()
}

fn response() -> Response<super::InProgress> {
    Response::start(
        ResponseId(Uuid::from_u128(3)),
        &opened(None),
        Some(MemberId(Uuid::from_u128(4))),
        None,
        at(2),
    )
    .unwrap()
}

fn answers(entries: Vec<(u128, Answer)>) -> Answers {
    entries.into_iter().map(|(n, a)| (qid(n), a)).collect()
}

fn completed() -> super::Response<super::Completed> {
    let graph = opened(None);
    let mut response = response();
    response
        .save(
            &graph,
            answers(vec![
                (1, Answer::Choice("Oui".to_string())),
                (3, Answer::Text("Très bien".to_string())),
            ]),
        )
        .unwrap();
    response.complete(&graph, at(3)).unwrap()
}

fn keys(value: &Value) -> Vec<String> {
    let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

#[test]
fn survey_opened_payload_is_locked() {
    let survey = opened(None);
    let event = SurveyOpened::new(&survey, None).unwrap();
    assert_eq!(event.name(), "survey.opened");
    assert_eq!(event.version(), 1);
    assert_eq!(event.subject().kind, "survey");
    assert_eq!(event.subject().id, Some(Uuid::from_u128(1)));
    assert_eq!(
        event.payload(),
        json!({
            "survey_id": Uuid::from_u128(1),
            "campaign_id": null,
            "metadata": { "theme": "survey" },
            "survey": {
                "id": Uuid::from_u128(1),
                "organization_id": Uuid::from_u128(900),
                "title": "Satisfaction",
                "description": "Après chantier",
                "visibility": "MEMBERS",
                "allow_multiple_responses": false,
                "response_cap": null,
                "opened_at": "2026-03-01T12:00:00Z",
            },
        })
    );
}

#[test]
fn survey_opened_payload_carries_the_campaign_id_and_merged_metadata() {
    let campaign = campaign();
    let survey = opened(Some(campaign.id()));
    let payload = SurveyOpened::new(&survey, Some(&campaign))
        .unwrap()
        .payload();
    assert_eq!(payload["campaign_id"], json!(Uuid::from_u128(2)));
    assert_eq!(
        payload["metadata"],
        json!({ "season": "spring", "theme": "survey" })
    );
}

#[test]
fn survey_opened_lets_the_survey_win_a_conflicting_key() {
    let campaign = conflicting_campaign();
    let survey = opened(Some(campaign.id()));
    let payload = SurveyOpened::new(&survey, Some(&campaign))
        .unwrap()
        .payload();
    assert_eq!(
        payload["metadata"],
        json!({ "season": "spring", "theme": "survey" })
    );
}

#[test]
fn survey_opened_refuses_a_campaign_that_is_not_the_survey_one() {
    let standalone = opened(None);
    let in_campaign = opened(Some(CampaignId(Uuid::from_u128(2))));
    assert_eq!(
        SurveyOpened::new(&standalone, Some(&campaign())).err(),
        Some(SurveyError::SurveyCampaignMismatch)
    );
    assert_eq!(
        SurveyOpened::new(&in_campaign, None).err(),
        Some(SurveyError::SurveyCampaignMismatch)
    );
    assert_eq!(
        SurveyOpened::new(&in_campaign, Some(&foreign_campaign())).err(),
        Some(SurveyError::SurveyCampaignMismatch)
    );
}

#[test]
fn response_submitted_payload_is_locked() {
    let survey = opened(None);
    let mut response = response();
    response
        .save(
            &survey,
            answers(vec![(1, Answer::Choice("Oui".to_string()))]),
        )
        .unwrap();
    let event = SurveyResponseSubmitted::new(&survey, None, &response).unwrap();
    assert_eq!(event.name(), "survey.response_submitted");
    assert_eq!(event.subject().kind, "survey_response");
    assert_eq!(event.subject().id, Some(Uuid::from_u128(3)));
    assert_eq!(
        event.payload(),
        json!({
            "survey_id": Uuid::from_u128(1),
            "campaign_id": null,
            "metadata": { "theme": "survey" },
            "response": {
                "id": Uuid::from_u128(3),
                "member_id": Uuid::from_u128(4),
                "status": "IN_PROGRESS",
            },
            "answered_questions": 1,
        })
    );
}

#[test]
fn response_submitted_lets_the_survey_win_a_conflicting_key() {
    let campaign = conflicting_campaign();
    let survey = opened(Some(campaign.id()));
    let response = response();
    let payload = SurveyResponseSubmitted::new(&survey, Some(&campaign), &response)
        .unwrap()
        .payload();
    assert_eq!(payload["campaign_id"], json!(Uuid::from_u128(2)));
    assert_eq!(
        payload["metadata"],
        json!({ "season": "spring", "theme": "survey" })
    );
}

#[test]
fn response_submitted_refuses_a_campaign_that_is_not_the_survey_one() {
    let survey = opened(None);
    let response = response();
    assert_eq!(
        SurveyResponseSubmitted::new(&survey, Some(&campaign()), &response).err(),
        Some(SurveyError::SurveyCampaignMismatch)
    );
}

#[test]
fn response_completed_payload_lists_only_the_path_in_order_with_labels() {
    let survey = opened(None);
    let mut response = response();
    response
        .save(
            &survey,
            answers(vec![
                (1, Answer::Choice("Oui".to_string())),
                (3, Answer::Text("Très bien".to_string())),
                (4, Answer::Text("hors chemin".to_string())),
            ]),
        )
        .unwrap();
    let completed = response.complete(&survey, at(3)).unwrap();
    let event = SurveyResponseCompleted::new(&survey, None, &completed).unwrap();
    assert_eq!(event.name(), "survey.response_completed");
    assert_eq!(
        event.payload(),
        json!({
            "survey_id": Uuid::from_u128(1),
            "campaign_id": null,
            "survey": { "id": Uuid::from_u128(1), "title": "Satisfaction" },
            "response": {
                "id": Uuid::from_u128(3),
                "member_id": Uuid::from_u128(4),
                "completed_at": "2026-03-03T12:00:00Z",
                "identifier_value": null,
            },
            "metadata": { "theme": "survey" },
            "answers": [
                { "question_id": Uuid::from_u128(1), "label": "Question 1", "kind": "SINGLE_CHOICE", "value": "Oui" },
                { "question_id": Uuid::from_u128(3), "label": "Question 3", "kind": "TEXT", "value": "Très bien" },
                { "question_id": Uuid::from_u128(5), "label": "Question 5", "kind": "NUMBER", "value": null },
            ],
        })
    );
}

#[test]
fn response_completed_payload_carries_merged_metadata() {
    let campaign = campaign();
    let survey = opened(Some(campaign.id()));
    let completed = completed();
    let payload = SurveyResponseCompleted::new(&survey, Some(&campaign), &completed)
        .unwrap()
        .payload();
    assert_eq!(payload["campaign_id"], json!(Uuid::from_u128(2)));
    assert_eq!(
        payload["metadata"],
        json!({ "season": "spring", "theme": "survey" })
    );
}

#[test]
fn response_completed_lets_the_survey_win_a_conflicting_key() {
    let campaign = conflicting_campaign();
    let survey = opened(Some(campaign.id()));
    let completed = completed();
    let payload = SurveyResponseCompleted::new(&survey, Some(&campaign), &completed)
        .unwrap()
        .payload();
    assert_eq!(
        payload["metadata"],
        json!({ "season": "spring", "theme": "survey" })
    );
}

#[test]
fn response_completed_refuses_a_campaign_that_is_not_the_survey_one() {
    let survey = opened(Some(CampaignId(Uuid::from_u128(2))));
    let completed = completed();
    assert_eq!(
        SurveyResponseCompleted::new(&survey, None, &completed).err(),
        Some(SurveyError::SurveyCampaignMismatch)
    );
}

#[test]
fn response_events_refuse_a_response_of_another_survey() {
    let mut other = Survey::new(
        SurveyId(Uuid::from_u128(5)),
        org(),
        "Autre".to_string(),
        Visibility::Members,
    )
    .unwrap();
    other.edit_graph(Some(graph()));
    let other = other.open(at(1), None).unwrap();
    let response = response();
    let completed = completed();
    assert_eq!(
        SurveyResponseSubmitted::new(&other, None, &response).err(),
        Some(SurveyError::ResponseSurveyMismatch)
    );
    assert_eq!(
        SurveyResponseCompleted::new(&other, None, &completed).err(),
        Some(SurveyError::ResponseSurveyMismatch)
    );
}

#[test]
fn answer_values_follow_their_kind() {
    let cases = [
        (Answer::Number(2.5), json!(2.5)),
        (
            Answer::Choices(vec!["a".to_string(), "b".to_string()]),
            json!(["a", "b"]),
        ),
        (
            Answer::Date(chrono::NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
            json!("2026-03-09"),
        ),
        (Answer::DateTime(at(4)), json!("2026-03-04T12:00:00Z")),
        (
            Answer::Email(super::EmailAddress::try_from("A@b.fr").unwrap()),
            json!("a@b.fr"),
        ),
        (
            Answer::Phone(super::PhoneNumber::try_from("06 12 34 56 78").unwrap()),
            json!("0612345678"),
        ),
        (Answer::Photos(vec!["k1".to_string()]), json!(["k1"])),
    ];
    for (answer, expected) in cases {
        assert_eq!(super::events::answer_value(&answer), expected);
    }
}

#[test]
fn campaign_started_payload_carries_campaign_id() {
    let event = SurveyCampaignStarted {
        campaign: campaign(),
    };
    assert_eq!(event.name(), "survey_campaign.started");
    assert_eq!(event.subject().kind, "survey_campaign");
    assert_eq!(event.subject().id, Some(Uuid::from_u128(2)));
    assert_eq!(
        event.payload(),
        json!({
            "campaign_id": Uuid::from_u128(2),
            "campaign": {
                "id": Uuid::from_u128(2),
                "organization_id": Uuid::from_u128(900),
                "name": "Printemps",
                "starts_at": "2026-03-01T12:00:00Z",
                "ends_at": null,
                "metadata": { "season": "spring" },
            },
        })
    );
}

#[test]
fn campaign_ended_payload_is_named_ended() {
    let event = SurveyCampaignEnded {
        campaign: campaign(),
    };
    assert_eq!(event.name(), "survey_campaign.ended");
    assert_eq!(event.payload()["campaign_id"], json!(Uuid::from_u128(2)));
}

#[test]
fn every_emitted_event_has_a_descriptor() {
    let mut emitted = emitted_events();
    let mut described: Vec<(&'static str, u16)> = descriptors()
        .iter()
        .map(|descriptor| (descriptor.name, descriptor.version))
        .collect();
    emitted.sort_unstable();
    described.sort_unstable();
    assert_eq!(emitted.len(), 5);
    assert_eq!(emitted, described);
}

fn real_payloads() -> Vec<(&'static str, u16, Value)> {
    let campaign = campaign();
    let survey = opened(Some(campaign.id()));
    let mut in_progress = response();
    in_progress
        .save(
            &survey,
            answers(vec![(1, Answer::Choice("Oui".to_string()))]),
        )
        .unwrap();
    let completed = completed();
    let opened_event = SurveyOpened::new(&survey, Some(&campaign)).unwrap();
    let submitted = SurveyResponseSubmitted::new(&survey, Some(&campaign), &in_progress).unwrap();
    let completed_event =
        SurveyResponseCompleted::new(&survey, Some(&campaign), &completed).unwrap();
    let started = SurveyCampaignStarted {
        campaign: campaign.clone(),
    };
    let ended = SurveyCampaignEnded { campaign };
    vec![
        (
            opened_event.name(),
            opened_event.version(),
            opened_event.payload(),
        ),
        (submitted.name(), submitted.version(), submitted.payload()),
        (
            completed_event.name(),
            completed_event.version(),
            completed_event.payload(),
        ),
        (started.name(), started.version(), started.payload()),
        (ended.name(), ended.version(), ended.payload()),
    ]
}

#[test]
fn every_event_version_matches_its_descriptor_and_its_emitted_line() {
    let real = real_payloads();
    assert_eq!(real.len(), 5);
    let emitted = emitted_events();
    let descriptors = descriptors();
    for (name, version, _) in real {
        let line = emitted
            .iter()
            .find(|(emitted, _)| *emitted == name)
            .unwrap();
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.name == name)
            .unwrap();
        assert_eq!(version, line.1, "{name}");
        assert_eq!(version, descriptor.version, "{name}");
    }
}

#[test]
fn every_descriptor_example_has_the_keys_of_the_real_payload() {
    let real = real_payloads();
    for descriptor in descriptors() {
        let payload = &real
            .iter()
            .find(|(name, _, _)| *name == descriptor.name)
            .unwrap()
            .2;
        assert_eq!(
            keys(&descriptor.payload_example),
            keys(payload),
            "{}",
            descriptor.name
        );
        for nested in ["survey", "response", "campaign", "metadata"] {
            if let Some(example) = descriptor.payload_example.get(nested) {
                if nested == "metadata" {
                    assert!(example.is_object(), "{} metadata", descriptor.name);
                    continue;
                }
                assert_eq!(
                    keys(example),
                    keys(&payload[nested]),
                    "{} {nested}",
                    descriptor.name
                );
            }
        }
    }
}

#[test]
fn the_completed_example_is_a_members_mode_response() {
    let example = descriptors()
        .into_iter()
        .find(|descriptor| descriptor.name == "survey.response_completed")
        .unwrap()
        .payload_example;
    assert!(example["response"]["member_id"].is_string());
    assert!(example["response"]["identifier_value"].is_null());
    let opened = descriptors()
        .into_iter()
        .find(|descriptor| descriptor.name == "survey.opened")
        .unwrap()
        .payload_example;
    assert_eq!(opened["survey"]["visibility"], json!("MEMBERS"));
    assert!(opened["survey"].get("metadata").is_none());
}
