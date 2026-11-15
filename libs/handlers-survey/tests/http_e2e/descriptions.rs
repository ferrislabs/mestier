use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

use crate::harness::{self, END, QUESTION};

const OTHER: &str = "0195f0a0-0000-7000-8000-0000000000d1";

fn described_graph(first: Value, second: Value) -> Value {
    json!({
        "nodes": [
            {"type": "QUESTION", "id": QUESTION, "kind": "TEXT", "label": "Votre nom", "required": true, "description": first},
            {"type": "QUESTION", "id": OTHER, "kind": "TEXT", "label": "Votre ville", "required": false, "description": second},
            {"type": "END", "id": END}
        ],
        "edges": [
            {"from": QUESTION, "to": OTHER, "branch": "NEXT"},
            {"from": OTHER, "to": END, "branch": "NEXT"}
        ],
        "start": QUESTION
    })
}

fn description_of(graph: &Value, id: &str) -> Value {
    graph["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|node| node["id"] == id))
        .map(|node| node["description"].clone())
        .unwrap_or(Value::String("missing".to_owned()))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_description_is_normalised_stored_and_shown_to_the_respondent() {
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
            Some(json!({"graph": described_graph(json!("x".repeat(501)), Value::Null)})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(
        refused["details"],
        json!({"reason": "graph_question_description_too_long", "question_id": QUESTION}),
        "{refused}"
    );

    let (status, saved) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            Some(json!({"graph": described_graph(
                json!("  Tel qu'inscrit sur le contrat\nsans abréviation  "),
                json!("   ")
            )})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(
        description_of(&saved["data"]["graph"], QUESTION),
        json!("Tel qu'inscrit sur le contrat\nsans abréviation")
    );
    assert_eq!(description_of(&saved["data"]["graph"], OTHER), Value::Null);

    let (status, read) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(
        description_of(&read["data"]["graph"], QUESTION),
        json!("Tel qu'inscrit sur le contrat\nsans abréviation")
    );

    let (status, opened) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{survey}/open")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{opened}");

    let (status, shown) = app
        .call(
            Method::GET,
            &app.org(&format!("/member-surveys/{survey}")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{shown}");
    assert_eq!(
        description_of(&shown["data"]["graph"], QUESTION),
        json!("Tel qu'inscrit sur le contrat\nsans abréviation")
    );
    assert_eq!(description_of(&shown["data"]["graph"], OTHER), Value::Null);

    app.cleanup().await;
}
