use reqwest::{Method, StatusCode};
use serde_json::json;

use crate::harness;

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_manager_creates_lists_reschedules_and_deletes_a_campaign() {
    let app = harness::start().await;
    let manager = Some(app.manager.as_str());

    let (status, created) = app
        .call(
            Method::POST,
            &app.org("/survey-campaigns"),
            manager,
            Some(json!({
                "name": "Printemps",
                "window": {"starts_at": "2099-03-01T00:00:00Z", "ends_at": "2099-03-31T00:00:00Z"},
                "metadata": {"saison": "printemps"}
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["status"], "SCHEDULED", "{created}");
    assert_eq!(created["data"]["metadata"], json!({"saison": "printemps"}));
    let id = created["data"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();

    let (status, listed) = app
        .call(
            Method::GET,
            &app.org("/survey-campaigns"),
            Some(&app.viewer),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["pagination"]["total"], 1, "{listed}");

    let (status, updated) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/survey-campaigns/{id}")),
            manager,
            Some(json!({"name": "Printemps 2099"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["data"]["name"], "Printemps 2099", "{updated}");

    let (status, answer) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/survey-campaigns/{id}")),
            manager,
            Some(json!({"window": {"starts_at": "2099-04-01T00:00:00Z", "ends_at": "2099-03-01T00:00:00Z"}})),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "an inverted window: {answer}"
    );

    let (status, answer) = app
        .call(
            Method::PATCH,
            &app.other_org(&format!("/survey-campaigns/{id}")),
            Some(&app.other_manager),
            Some(json!({"name": "Volé"})),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "another organization's campaign: {answer}"
    );

    let (status, read) = app
        .call(
            Method::GET,
            &app.org(&format!("/survey-campaigns/{id}")),
            Some(&app.viewer),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(read["data"]["id"], id.as_str(), "{read}");
    assert_eq!(read["data"]["name"], "Printemps 2099", "{read}");

    let (status, answer) = app
        .call(
            Method::GET,
            &app.org(&format!("/survey-campaigns/{id}")),
            Some(&app.member),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{answer}");

    for method in [Method::GET, Method::DELETE] {
        let (status, answer) = app
            .call(
                method,
                &app.other_org(&format!("/survey-campaigns/{id}")),
                Some(&app.other_manager),
                None,
            )
            .await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "another organization's campaign: {answer}"
        );
    }

    let (status, answer) = app
        .call(
            Method::DELETE,
            &app.org(&format!("/survey-campaigns/{id}")),
            Some(&app.member),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{answer}");

    let (status, _) = app
        .call(
            Method::DELETE,
            &app.org(&format!("/survey-campaigns/{id}")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    app.cleanup().await;
}
