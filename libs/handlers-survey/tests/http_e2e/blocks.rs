use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

use crate::harness::{self, END, QUESTION};

const CONDITION: &str = "0195f0a0-0000-7000-8000-0000000000c1";
const DATE: &str = "0195f0a0-0000-7000-8000-0000000000c2";
const PLACE: &str = "0195f0a0-0000-7000-8000-0000000000c3";
const BLOCK: &str = "0195f0a0-0000-7000-8000-0000000000b1";

fn blocked_graph(place_block: Value) -> Value {
    json!({
        "nodes": [
            {"type": "QUESTION", "id": QUESTION, "kind": "SINGLE_CHOICE", "label": "Le stage vous a-t-il plu ?", "required": true, "options": ["Oui", "Non"]},
            {"type": "CONDITION", "id": CONDITION, "question": QUESTION, "option": "Oui", "checked": true},
            {"type": "QUESTION", "id": DATE, "kind": "TEXT", "label": "Quand ?", "required": true, "block": BLOCK},
            {"type": "QUESTION", "id": PLACE, "kind": "TEXT", "label": "Où ?", "required": true, "block": place_block},
            {"type": "END", "id": END}
        ],
        "edges": [
            {"from": QUESTION, "to": CONDITION, "branch": "NEXT"},
            {"from": CONDITION, "to": DATE, "branch": "THEN"},
            {"from": CONDITION, "to": END, "branch": "ELSE"},
            {"from": DATE, "to": PLACE, "branch": "NEXT"},
            {"from": PLACE, "to": END, "branch": "NEXT"}
        ],
        "start": QUESTION
    })
}

fn block_of(graph: &Value, id: &str) -> Value {
    graph["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|node| node["id"] == id))
        .map(|node| node["block"].clone())
        .unwrap_or(Value::String("missing".to_owned()))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_block_is_stored_shown_to_the_respondent_and_answered_question_by_question() {
    let app = harness::start().await;
    let manager = Some(app.manager.as_str());
    let member = Some(app.member.as_str());
    let survey = app
        .draft_survey(|s| app.org(s), &app.manager, "MEMBERS")
        .await;

    let (status, refused) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            Some(json!({"graph": blocked_graph(Value::Null)})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(
        refused["details"],
        json!({"reason": "graph_block_too_small", "question_id": DATE}),
        "{refused}"
    );

    let (status, saved) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            Some(json!({"graph": blocked_graph(json!(BLOCK))})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(block_of(&saved["data"]["graph"], DATE), json!(BLOCK));
    assert_eq!(block_of(&saved["data"]["graph"], QUESTION), Value::Null);
    let (status, opened) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{survey}/open")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{opened}");

    let (status, read) = app
        .call(
            Method::GET,
            &app.org(&format!("/member-surveys/{survey}")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(block_of(&read["data"]["graph"], PLACE), json!(BLOCK));

    let (status, started) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-surveys/{survey}/responses")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    let response = started["data"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let (status, partial) = app
        .call(
            Method::PUT,
            &app.org(&format!("/member-survey-responses/{response}")),
            member,
            Some(json!({"answers": {QUESTION: {"Choice": "Oui"}, DATE: {"Text": "Lundi"}}})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{partial}");
    let (status, incomplete) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-survey-responses/{response}/complete")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{incomplete}");
    assert_eq!(
        incomplete["details"],
        json!({"reason": "required_question_unanswered", "question_id": PLACE}),
        "{incomplete}"
    );
    let (status, whole) = app
        .call(
            Method::PUT,
            &app.org(&format!("/member-survey-responses/{response}")),
            member,
            Some(json!({"answers": {
                QUESTION: {"Choice": "Oui"},
                DATE: {"Text": "Lundi"},
                PLACE: {"Text": "Lyon"}
            }})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{whole}");
    let (status, completed) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-survey-responses/{response}/complete")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{completed}");

    app.cleanup().await;
}
