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
    SurveyId, Visibility, Window,
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
    draft(campaign).open(at(1)).unwrap()
}

fn campaign() -> Campaign {
    Campaign {
        id: CampaignId(Uuid::from_u128(2)),
        organization_id: org(),
        name: "Printemps".to_string(),
        window: Window::new(Some(at(1)), None).unwrap(),
        metadata: metadata(&[("season", "spring")]),
    }
}

fn response() -> Response<super::InProgress> {
    Response::start(
        ResponseId(Uuid::from_u128(3)),
        SurveyId(Uuid::from_u128(1)),
        org(),
        Some(MemberId(Uuid::from_u128(4))),
        None,
        at(2),
    )
}

fn answers(entries: Vec<(u128, Answer)>) -> Answers {
    entries.into_iter().map(|(n, a)| (qid(n), a)).collect()
}

fn completed() -> super::Response<super::Completed> {
    let graph = graph();
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
    let event = SurveyOpened {
        survey: opened(None),
    };
    assert_eq!(event.name(), "survey.opened");
    assert_eq!(event.version(), 1);
    assert_eq!(event.subject().kind, "survey");
    assert_eq!(event.subject().id, Some(Uuid::from_u128(1)));
    assert_eq!(
        event.payload(),
        json!({
            "survey_id": Uuid::from_u128(1),
            "campaign_id": null,
            "survey": {
                "id": Uuid::from_u128(1),
                "organization_id": Uuid::from_u128(900),
                "title": "Satisfaction",
                "description": "Après chantier",
                "visibility": "MEMBERS",
                "allow_multiple_responses": false,
                "response_cap": null,
                "metadata": { "theme": "survey" },
                "opened_at": "2026-03-01T12:00:00Z",
            },
        })
    );
}

#[test]
fn survey_opened_payload_carries_the_campaign_id() {
    let event = SurveyOpened {
        survey: opened(Some(CampaignId(Uuid::from_u128(2)))),
    };
    assert_eq!(event.payload()["campaign_id"], json!(Uuid::from_u128(2)));
}

#[test]
fn response_submitted_payload_is_locked() {
    let mut response = response();
    response
        .save(
            &graph(),
            answers(vec![(1, Answer::Choice("Oui".to_string()))]),
        )
        .unwrap();
    let event = SurveyResponseSubmitted {
        response,
        campaign_id: None,
    };
    assert_eq!(event.name(), "survey.response_submitted");
    assert_eq!(event.subject().kind, "survey_response");
    assert_eq!(event.subject().id, Some(Uuid::from_u128(3)));
    assert_eq!(
        event.payload(),
        json!({
            "survey_id": Uuid::from_u128(1),
            "campaign_id": null,
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
fn response_completed_payload_lists_only_the_path_in_order_with_labels() {
    let graph = graph();
    let mut response = response();
    response
        .save(
            &graph,
            answers(vec![
                (1, Answer::Choice("Oui".to_string())),
                (3, Answer::Text("Très bien".to_string())),
                (4, Answer::Text("hors chemin".to_string())),
            ]),
        )
        .unwrap();
    let completed = response.complete(&graph, at(3)).unwrap();
    let event = SurveyResponseCompleted {
        response: completed,
        survey: opened(None),
        metadata: Metadata::default(),
    };
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
            "metadata": {},
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
    let survey = opened(Some(campaign.id));
    let merged = Metadata::merged(&campaign.metadata, survey.metadata());
    let event = SurveyResponseCompleted {
        response: completed(),
        survey,
        metadata: merged,
    };
    let payload = event.payload();
    assert_eq!(payload["campaign_id"], json!(Uuid::from_u128(2)));
    assert_eq!(
        payload["metadata"],
        json!({ "season": "spring", "theme": "survey" })
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
    assert_eq!(event.version(), 1);
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

#[test]
fn every_descriptor_example_has_the_keys_of_the_real_payload() {
    let campaign = campaign();
    let survey = opened(Some(campaign.id));
    let mut in_progress = response();
    in_progress
        .save(
            &graph(),
            answers(vec![(1, Answer::Choice("Oui".to_string()))]),
        )
        .unwrap();
    let real: Vec<(&str, Value)> = vec![
        (
            "survey.opened",
            SurveyOpened {
                survey: survey.clone(),
            }
            .payload(),
        ),
        (
            "survey.response_submitted",
            SurveyResponseSubmitted {
                response: in_progress,
                campaign_id: Some(campaign.id),
            }
            .payload(),
        ),
        (
            "survey.response_completed",
            SurveyResponseCompleted {
                response: completed(),
                survey,
                metadata: Metadata::default(),
            }
            .payload(),
        ),
        (
            "survey_campaign.started",
            SurveyCampaignStarted {
                campaign: campaign.clone(),
            }
            .payload(),
        ),
        (
            "survey_campaign.ended",
            SurveyCampaignEnded { campaign }.payload(),
        ),
    ];
    for descriptor in descriptors() {
        let payload = &real
            .iter()
            .find(|(name, _)| *name == descriptor.name)
            .unwrap()
            .1;
        assert_eq!(
            keys(&descriptor.payload_example),
            keys(payload),
            "{}",
            descriptor.name
        );
        for nested in ["survey", "response", "campaign"] {
            if let Some(example) = descriptor.payload_example.get(nested) {
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
