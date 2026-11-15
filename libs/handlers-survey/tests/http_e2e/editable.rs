use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

use crate::harness::{self, App, END, QUESTION};

const OTHER_QUESTION: &str = "0195f0a0-0000-7000-8000-0000000000aa";

fn other_graph() -> Value {
    json!({
        "nodes": [
            {"type": "QUESTION", "id": OTHER_QUESTION, "kind": "TEXT", "label": "Un commentaire ?", "required": false},
            {"type": "END", "id": END}
        ],
        "edges": [{"from": OTHER_QUESTION, "to": END, "branch": "NEXT"}],
        "start": OTHER_QUESTION
    })
}

fn id_of(body: &Value) -> String {
    body["data"]["id"].as_str().unwrap_or_default().to_owned()
}

async fn campaign(app: &App, name: &str) -> String {
    let (status, created) = app
        .call(
            Method::POST,
            &app.org("/survey-campaigns"),
            Some(&app.manager),
            Some(json!({"name": name, "window": {}})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    id_of(&created)
}

async fn complete_as_member(app: &App, survey: &str) -> Value {
    let member = Some(app.member.as_str());
    let (status, started) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-surveys/{survey}/responses")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    let response = id_of(&started);
    let (status, saved) = app
        .call(
            Method::PUT,
            &app.org(&format!("/member-survey-responses/{response}")),
            member,
            Some(json!({"answers": {QUESTION: {"Choice": "Oui"}}})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (status, completed) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-survey-responses/{response}/complete")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{completed}");
    completed
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_opened_survey_is_edited_versioned_and_shared_by_campaigns() {
    let app = harness::start().await;
    let manager = Some(app.manager.as_str());
    let spring = campaign(&app, "Printemps").await;
    let autumn = campaign(&app, "Automne").await;

    let (status, created) = app
        .call(
            Method::POST,
            &app.org("/surveys"),
            manager,
            Some(json!({"title": "Bilan", "visibility": "MEMBERS", "campaign_ids": [autumn]})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(
        created["data"]["campaign_ids"],
        json!([autumn]),
        "{created}"
    );
    assert_eq!(created["data"]["graph_version"], json!(null), "{created}");
    let survey = id_of(&created);

    let (status, drafted) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            Some(json!({"graph": harness::graph(), "campaign_ids": [spring, autumn]})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{drafted}");
    assert_eq!(drafted["data"]["graph_version"], 1, "{drafted}");
    assert_eq!(drafted["data"]["campaign_ids"], json!([spring, autumn]));
    let (status, _) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{survey}/open")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, second) = app
        .call(
            Method::POST,
            &app.org("/surveys"),
            manager,
            Some(json!({"title": "Second", "visibility": "PUBLIC", "campaign_ids": [spring]})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{second}");
    let (status, held) = app
        .call(
            Method::GET,
            &app.org(&format!("/survey-campaigns/{spring}/surveys")),
            Some(&app.viewer),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{held}");
    let mut titles: Vec<&str> = held["data"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["title"].as_str())
                .collect()
        })
        .unwrap_or_default();
    titles.sort_unstable();
    assert_eq!(titles, ["Bilan", "Second"], "{held}");
    let (status, filtered) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys?campaign_id={autumn}")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{filtered}");
    assert_eq!(filtered["pagination"]["total"], 1, "{filtered}");
    assert_eq!(filtered["data"][0]["id"], survey.as_str(), "{filtered}");

    let completed = complete_as_member(&app, &survey).await;
    let response = id_of(&completed);
    let (status, edited) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            Some(json!({"title": "Bilan revu", "graph": other_graph(), "campaign_ids": []})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{edited}");
    assert_eq!(edited["data"]["status"], "OPENED", "{edited}");
    assert_eq!(edited["data"]["title"], "Bilan revu", "{edited}");
    assert_eq!(edited["data"]["graph_version"], 2, "{edited}");
    assert_eq!(edited["data"]["campaign_ids"], json!([]), "{edited}");

    let (status, read) = app
        .call(
            Method::GET,
            &app.org(&format!("/survey-responses/{response}")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(read["data"]["graph_version"], 1, "{read}");
    let (status, former) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{survey}/graph-versions/1")),
            Some(&app.viewer),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{former}");
    assert_eq!(former["data"]["version"], 1, "{former}");
    assert_eq!(former["data"]["graph"]["start"], QUESTION, "{former}");
    let (status, current) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{survey}/graph-versions/2")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{current}");
    assert_eq!(
        current["data"]["graph"]["start"], OTHER_QUESTION,
        "{current}"
    );
    let (status, _) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{survey}/graph-versions/3")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, frozen) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            Some(json!({"visibility": "PUBLIC"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{frozen}");
    assert_eq!(frozen["details"]["reason"], "survey_visibility_frozen");

    let (status, closed) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{survey}/close")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{closed}");
    let (status, retitled) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{survey}")),
            manager,
            Some(json!({"title": "Bilan clos"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{retitled}");
    assert_eq!(retitled["data"]["status"], "CLOSED", "{retitled}");

    app.cleanup().await;
}
