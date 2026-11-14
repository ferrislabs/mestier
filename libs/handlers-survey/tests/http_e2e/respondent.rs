use std::collections::BTreeSet;

use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

use crate::harness::{self, App, QUESTION};

fn keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .map(|object| object.keys().map(String::as_str).collect())
        .unwrap_or_default()
}

fn never_leaks(body: &Value) {
    let text = body.to_string();
    for hidden in [
        "organization_id",
        "metadata",
        "allow_multiple_responses",
        "response_cap",
        "campaign_id",
        "resume_token",
        "member_id",
        "promotion",
    ] {
        assert!(
            !text.contains(hidden),
            "a respondent route exposes {hidden}: {text}"
        );
    }
}

async fn opened_with_metadata(app: &App) -> String {
    let id = app
        .draft_survey(|s| app.org(s), &app.manager, "MEMBERS")
        .await;
    let (status, body) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{id}")),
            Some(&app.manager),
            Some(json!({"metadata": {"promotion": "2026"}, "response_cap": 5})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{id}/open")),
            Some(&app.manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    id
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_answers_a_members_survey_and_only_sees_what_a_respondent_needs() {
    let app = harness::start().await;
    let member = Some(app.member.as_str());
    let survey = opened_with_metadata(&app).await;

    let (status, read) = app
        .call(
            Method::GET,
            &app.org(&format!("/member-surveys/{survey}")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(
        keys(&read["data"]),
        BTreeSet::from(["id", "title", "description", "visibility", "graph"]),
        "{read}"
    );
    never_leaks(&read);

    let (status, started) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-surveys/{survey}/responses")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    assert_eq!(
        keys(&started["data"]),
        BTreeSet::from(["id", "status", "answers"]),
        "{started}"
    );
    assert_eq!(started["data"]["status"], "IN_PROGRESS", "{started}");
    never_leaks(&started);
    let response = started["data"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();

    let (status, resumed) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-surveys/{survey}/responses")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{resumed}");
    assert_eq!(
        resumed["data"]["id"],
        response.as_str(),
        "a second start resumes: {resumed}"
    );

    let (status, early) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-survey-responses/{response}/complete")),
            member,
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a required question is unanswered: {early}"
    );

    for answers in [
        json!({QUESTION: {"Choice": "Peut-être"}}),
        json!({QUESTION: {"Text": "Oui"}}),
        json!({"0195f0a0-0000-7000-8000-0000000000ff": {"Choice": "Oui"}}),
        json!({QUESTION: {"Email": "pas un e-mail"}}),
    ] {
        let (status, refused) = app
            .call(
                Method::PUT,
                &app.org(&format!("/member-survey-responses/{response}")),
                member,
                Some(json!({"answers": answers})),
            )
            .await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{answers}: {refused}"
        );
    }

    let (status, saved) = app
        .call(
            Method::PUT,
            &app.org(&format!("/member-survey-responses/{response}")),
            member,
            Some(json!({"answers": {QUESTION: {"Choice": "Oui"}}})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(
        saved["data"]["answers"][QUESTION],
        json!({"Choice": "Oui"}),
        "{saved}"
    );
    never_leaks(&saved);

    let (status, completed) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-survey-responses/{response}/complete")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{completed}");
    assert_eq!(completed["data"]["status"], "COMPLETED", "{completed}");
    never_leaks(&completed);

    let (status, again) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-survey-responses/{response}/complete")),
            member,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{again}");
    let (status, again) = app
        .call(
            Method::PUT,
            &app.org(&format!("/member-survey-responses/{response}")),
            member,
            Some(json!({"answers": {QUESTION: {"Choice": "Non"}}})),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a complete response is frozen: {again}"
    );
    let (status, again) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-surveys/{survey}/responses")),
            member,
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "one response per member: {again}"
    );

    let (status, listed) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{survey}/responses")),
            Some(&app.viewer),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["pagination"]["total"], 1, "{listed}");
    assert_eq!(listed["data"][0]["status"], "COMPLETED", "{listed}");
    assert!(listed["data"][0]["member_id"].is_string(), "{listed}");

    let (status, one) = app
        .call(
            Method::GET,
            &app.org(&format!("/survey-responses/{response}")),
            Some(&app.manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{one}");
    assert_eq!(
        one["data"]["answers"][QUESTION],
        json!({"Choice": "Oui"}),
        "{one}"
    );

    let (status, _) = app
        .call(
            Method::DELETE,
            &app.org(&format!("/survey-responses/{response}")),
            Some(&app.viewer),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = app
        .call(
            Method::DELETE,
            &app.org(&format!("/survey-responses/{response}")),
            Some(&app.manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_of_another_organization_cannot_answer_and_a_foreign_survey_is_not_found() {
    let app = harness::start().await;
    let survey = app.opened_survey(|s| app.org(s), &app.manager).await;
    let foreign = app
        .opened_survey(|s| app.other_org(s), &app.other_manager)
        .await;

    for (method, path) in [
        (Method::GET, app.org(&format!("/member-surveys/{survey}"))),
        (
            Method::POST,
            app.org(&format!("/member-surveys/{survey}/responses")),
        ),
    ] {
        let (status, answer) = app.call(method, &path, Some(&app.other_member), None).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}: {answer}");
    }

    for (method, path) in [
        (Method::GET, app.org(&format!("/member-surveys/{foreign}"))),
        (
            Method::POST,
            app.org(&format!("/member-surveys/{foreign}/responses")),
        ),
    ] {
        let (status, answer) = app.call(method, &path, Some(&app.member), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}: {answer}");
    }

    let (status, started) = app
        .call(
            Method::POST,
            &app.other_org(&format!("/member-surveys/{foreign}/responses")),
            Some(&app.other_member),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{started}");
    let foreign_response = started["data"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    for (method, path, body) in [
        (
            Method::PUT,
            app.org(&format!("/member-survey-responses/{foreign_response}")),
            Some(json!({"answers": {QUESTION: {"Choice": "Oui"}}})),
        ),
        (
            Method::POST,
            app.org(&format!(
                "/member-survey-responses/{foreign_response}/complete"
            )),
            None,
        ),
        (
            Method::GET,
            app.org(&format!("/survey-responses/{foreign_response}")),
            None,
        ),
    ] {
        let token = if method == Method::GET {
            &app.manager
        } else {
            &app.member
        };
        let (status, answer) = app.call(method, &path, Some(token), body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}: {answer}");
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_draft_closed_or_flag_off_survey_is_refused_to_its_respondents() {
    let app = harness::start().await;
    let draft = app
        .draft_survey(|s| app.org(s), &app.manager, "MEMBERS")
        .await;
    let closed = app.opened_survey(|s| app.org(s), &app.manager).await;
    let (status, body) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{closed}/close")),
            Some(&app.manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, answer) = app
        .call(
            Method::GET,
            &app.org(&format!("/member-surveys/{draft}")),
            Some(&app.member),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{answer}");
    let (status, answer) = app
        .call(
            Method::GET,
            &app.org(&format!("/member-surveys/{closed}")),
            Some(&app.member),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");
    let (status, answer) = app
        .call(
            Method::POST,
            &app.org(&format!("/member-surveys/{closed}/responses")),
            Some(&app.member),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");

    let (status, answer) = app
        .call(
            Method::GET,
            &app.flag_off_org(&format!("/member-surveys/{draft}")),
            Some(&app.flag_off_manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{answer}");

    app.cleanup().await;
}
