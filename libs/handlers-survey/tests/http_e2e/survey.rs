use reqwest::{Method, StatusCode};
use serde_json::json;

use crate::harness::{self, END, QUESTION};

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_manager_drafts_edits_opens_duplicates_closes_and_deletes_a_survey() {
    let app = harness::start().await;
    let manager = Some(app.manager.as_str());

    let (status, created) = app
        .call(
            Method::POST,
            &app.org("/surveys"),
            manager,
            Some(json!({"title": "Bilan de stage", "visibility": "MEMBERS"})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["status"], "DRAFT", "{created}");
    assert_eq!(created["data"]["graph"], json!(null), "{created}");
    let id = created["data"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();

    let (status, updated) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{id}")),
            manager,
            Some(json!({
                "description": "Fin de session",
                "response_cap": 10,
                "metadata": {"promotion": "2026"},
                "graph": harness::graph(),
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["data"]["response_cap"], 10, "{updated}");
    assert_eq!(updated["data"]["metadata"], json!({"promotion": "2026"}));
    assert_eq!(updated["data"]["graph"]["start"], QUESTION, "{updated}");

    let (status, listed) = app
        .call(Method::GET, &app.org("/surveys"), manager, None)
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["pagination"]["total"], 1, "{listed}");
    assert_eq!(listed["data"][0]["id"], id.as_str(), "{listed}");

    let (status, viewed) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{id}")),
            Some(&app.viewer),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "a survey.view holder reads: {viewed}"
    );

    let (status, opened) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{id}/open")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{opened}");
    assert_eq!(opened["data"]["status"], "OPENED", "{opened}");
    assert!(opened["data"]["opened_at"].is_string(), "{opened}");

    let (status, copy) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{id}/duplicate")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{copy}");
    assert_eq!(copy["data"]["status"], "DRAFT", "{copy}");
    let copy_id = copy["data"]["id"].as_str().unwrap_or_default().to_owned();
    assert_ne!(copy_id, id);

    let (status, closed) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{id}/close")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{closed}");
    assert_eq!(closed["data"]["status"], "CLOSED", "{closed}");

    let (status, _) = app
        .call(
            Method::DELETE,
            &app.org(&format!("/surveys/{copy_id}")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{copy_id}")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn every_survey_route_family_refuses_a_caller_without_a_token() {
    let app = harness::start().await;
    let survey = "0195f0a0-0000-7000-8000-0000000000aa";

    for (method, path) in [
        (Method::GET, app.org("/surveys")),
        (Method::POST, app.org(&format!("/surveys/{survey}/open"))),
        (Method::GET, app.org("/survey-campaigns")),
        (
            Method::GET,
            app.org(&format!("/surveys/{survey}/responses")),
        ),
        (Method::GET, app.org(&format!("/member-surveys/{survey}"))),
        (
            Method::POST,
            app.org(&format!("/member-surveys/{survey}/responses")),
        ),
    ] {
        let (status, body) = app.call(method, &path, None, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}: {body}");
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_survey_manage_cannot_manage_surveys() {
    let app = harness::start().await;
    let id = app
        .draft_survey(|s| app.org(s), &app.manager, "MEMBERS")
        .await;

    for token in [&app.member, &app.viewer] {
        for (method, path, body) in [
            (
                Method::POST,
                app.org("/surveys"),
                Some(json!({"title": "X", "visibility": "MEMBERS"})),
            ),
            (
                Method::PATCH,
                app.org(&format!("/surveys/{id}")),
                Some(json!({"title": "Y"})),
            ),
            (Method::POST, app.org(&format!("/surveys/{id}/open")), None),
            (
                Method::POST,
                app.org(&format!("/surveys/{id}/duplicate")),
                None,
            ),
            (Method::DELETE, app.org(&format!("/surveys/{id}")), None),
        ] {
            let (status, answer) = app.call(method, &path, Some(token), body).await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{path}: {answer}");
        }
    }
    for (method, path) in [
        (Method::GET, app.org("/surveys")),
        (Method::GET, app.org(&format!("/surveys/{id}"))),
    ] {
        let (status, answer) = app.call(method, &path, Some(&app.member), None).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "reading needs survey.view: {path}: {answer}"
        );
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_survey_of_another_organization_is_not_found_and_its_path_is_forbidden() {
    let app = harness::start().await;
    let foreign = app
        .draft_survey(|s| app.other_org(s), &app.other_manager, "MEMBERS")
        .await;

    for (method, path) in [
        (Method::GET, app.org(&format!("/surveys/{foreign}"))),
        (Method::PATCH, app.org(&format!("/surveys/{foreign}"))),
        (Method::POST, app.org(&format!("/surveys/{foreign}/open"))),
        (Method::DELETE, app.org(&format!("/surveys/{foreign}"))),
    ] {
        let body = (method == Method::PATCH).then(|| json!({"title": "Pris"}));
        let (status, answer) = app.call(method, &path, Some(&app.manager), body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}: {answer}");
    }

    let (status, answer) = app
        .call(
            Method::GET,
            &app.other_org("/surveys"),
            Some(&app.manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{answer}");

    let (status, still) = app
        .call(
            Method::GET,
            &app.other_org(&format!("/surveys/{foreign}")),
            Some(&app.other_manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{still}");
    assert_eq!(still["data"]["title"], "Satisfaction", "{still}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_organization_without_the_surveys_flag_finds_no_survey_route() {
    let app = harness::start().await;
    let token = Some(app.flag_off_manager.as_str());

    for (method, path, body) in [
        (Method::GET, app.flag_off_org("/surveys"), None),
        (
            Method::POST,
            app.flag_off_org("/surveys"),
            Some(json!({"title": "X", "visibility": "MEMBERS"})),
        ),
        (Method::GET, app.flag_off_org("/survey-campaigns"), None),
        (
            Method::POST,
            app.flag_off_org("/survey-campaigns"),
            Some(json!({"name": "Printemps", "window": {"starts_at": null, "ends_at": null}})),
        ),
    ] {
        let (status, answer) = app.call(method, &path, token, body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}: {answer}");
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn invalid_survey_input_is_unprocessable_and_a_state_conflict_is_a_conflict() {
    let app = harness::start().await;
    let manager = Some(app.manager.as_str());

    let (status, answer) = app
        .call(
            Method::POST,
            &app.org("/surveys"),
            manager,
            Some(json!({"title": "   ", "visibility": "MEMBERS"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    assert_eq!(answer["code"], "E_UNPROCESSABLE_ENTITY", "{answer}");

    let (status, created) = app
        .call(
            Method::POST,
            &app.org("/surveys"),
            manager,
            Some(json!({"title": "Sans graphe", "visibility": "MEMBERS"})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["data"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();

    let cycle = json!({
        "nodes": [
            {"type": "QUESTION", "id": QUESTION, "kind": "TEXT", "label": "A", "required": false},
            {"type": "END", "id": END}
        ],
        "edges": [{"from": QUESTION, "to": QUESTION, "branch": "NEXT"}],
        "start": QUESTION
    });
    for body in [
        json!({"graph": cycle}),
        json!({"response_cap": 0}),
        json!({"metadata": {"promotion": " "}}),
        json!({"title": ""}),
    ] {
        let (status, answer) = app
            .call(
                Method::PATCH,
                &app.org(&format!("/surveys/{id}")),
                manager,
                Some(body.clone()),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}: {answer}");
    }

    let (status, answer) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{id}/open")),
            manager,
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a survey without a graph cannot open: {answer}"
    );

    let (status, answer) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{id}/close")),
            manager,
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a draft cannot close: {answer}"
    );

    app.cleanup().await;
}
