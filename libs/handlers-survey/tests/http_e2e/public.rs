use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App, QUESTION};

const PHOTO: &[u8] = include_bytes!("fixtures/chantier.jpg");

fn client_ip() -> String {
    let id = Uuid::now_v7().as_u128();
    format!("198.51.{}.{}", (id >> 8) as u8, id as u8)
}

fn json_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

async fn public_survey(app: &App) -> (String, String) {
    let id = app
        .opened_survey_as(|s| app.org(s), &app.manager, "PUBLIC")
        .await;
    let (status, body) = app
        .call(
            Method::GET,
            &app.org(&format!("/surveys/{id}")),
            Some(&app.manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let slug = body["data"]["public_slug"]
        .as_str()
        .expect("an opened public survey has a slug")
        .to_owned();
    (id, slug)
}

async fn start(app: &App, slug: &str, ip: &str) -> String {
    let (status, text) = app
        .send(
            Method::POST,
            &format!("/public/surveys/{slug}/responses"),
            ip,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    json_of(&text)["data"]["resume_token"]
        .as_str()
        .expect("the started response carries its resume token")
        .to_owned()
}

fn answers(question: &str, answer: Value) -> Option<(&'static str, Vec<u8>)> {
    Some((
        "application/json",
        serde_json::to_vec(&json!({"answers": {question: answer}})).unwrap(),
    ))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_public_survey_is_answered_without_an_account_and_never_shows_its_organization() {
    let app = harness::start().await;
    let ip = client_ip();
    let (_, slug) = public_survey(&app).await;
    let organization = app.organization_id.to_string();

    let (read_status, read) = app
        .send(Method::GET, &format!("/public/surveys/{slug}"), &ip, None)
        .await;
    let (start_status, started) = app
        .send(
            Method::POST,
            &format!("/public/surveys/{slug}/responses"),
            &ip,
            None,
        )
        .await;
    let token = json_of(&started)["data"]["resume_token"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let response = format!("/public/surveys/{slug}/responses/{token}");
    let (resume_status, resumed) = app.send(Method::GET, &response, &ip, None).await;
    let (save_status, saved) = app
        .send(
            Method::PUT,
            &response,
            &ip,
            answers(QUESTION, json!({"Choice": "Oui"})),
        )
        .await;
    let (complete_status, completed) = app
        .send(Method::POST, &format!("{response}/complete"), &ip, None)
        .await;
    let (again_status, again) = app.send(Method::GET, &response, &ip, None).await;

    assert_eq!(read_status, StatusCode::OK, "{read}");
    assert_eq!(start_status, StatusCode::CREATED, "{started}");
    assert!(token.len() >= 32, "{started}");
    assert_eq!(resume_status, StatusCode::OK, "{resumed}");
    assert_eq!(save_status, StatusCode::OK, "{saved}");
    assert_eq!(complete_status, StatusCode::OK, "{completed}");
    assert_eq!(json_of(&completed)["data"]["status"], "COMPLETED");
    assert_eq!(again_status, StatusCode::CONFLICT, "{again}");
    assert_eq!(
        json_of(&again)["details"]["reason"],
        "response_already_completed"
    );
    for body in [&read, &started, &resumed, &saved, &completed] {
        assert!(!body.contains(&organization), "{body}");
        for hidden in [
            "organization_id",
            "metadata",
            "campaign_id",
            "member_id",
            "response_cap",
        ] {
            assert!(!body.contains(hidden), "{hidden} in {body}");
        }
    }
    for body in [&read, &resumed, &saved, &completed] {
        assert!(!body.contains("resume_token"), "{body}");
        assert!(!body.contains(&token), "{body}");
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn every_hidden_survey_and_every_bad_token_answers_the_same_404_byte_for_byte() {
    let app = harness::start().await;
    let ip = client_ip();
    let (_, slug) = public_survey(&app).await;
    let token = start(&app, &slug, &ip).await;
    let flag_off = app
        .opened_survey_as(|s| app.other_org(s), &app.other_manager, "PUBLIC")
        .await;
    let (status, body) = app
        .call(
            Method::GET,
            &app.other_org(&format!("/surveys/{flag_off}")),
            Some(&app.other_manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let flag_off_slug = body["data"]["public_slug"].as_str().unwrap().to_owned();
    let (status, _) = app
        .send(
            Method::GET,
            &format!("/public/surveys/{flag_off_slug}"),
            &ip,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    app.set_surveys_flag(app.other_organization_id, false).await;
    let members = app.opened_survey(|s| app.org(s), &app.manager).await;
    let draft = app
        .draft_survey(|s| app.org(s), &app.manager, "PUBLIC")
        .await;
    let unknown = "inconnu-00000000000000";
    let photo = Some(("image/jpeg", PHOTO.to_vec()));

    let (reference_status, reference) = app
        .send(
            Method::GET,
            &format!("/public/surveys/{unknown}"),
            &ip,
            None,
        )
        .await;
    let attempts = [
        (Method::GET, "/public/surveys/x".to_owned(), None),
        (
            Method::GET,
            format!("/public/surveys/{}", "%20".repeat(9)),
            None,
        ),
        (
            Method::GET,
            format!("/public/surveys/{flag_off_slug}"),
            None,
        ),
        (Method::GET, format!("/public/surveys/{members}"), None),
        (Method::GET, format!("/public/surveys/{draft}"), None),
        (
            Method::POST,
            format!("/public/surveys/{unknown}/responses"),
            None,
        ),
        (
            Method::POST,
            format!("/public/surveys/{flag_off_slug}/responses"),
            None,
        ),
        (
            Method::POST,
            format!("/public/surveys/{members}/responses"),
            None,
        ),
        (
            Method::GET,
            format!("/public/surveys/{slug}/responses/{}", "y".repeat(43)),
            None,
        ),
        (
            Method::GET,
            format!("/public/surveys/{unknown}/responses/{token}"),
            None,
        ),
        (
            Method::GET,
            format!("/public/surveys/{flag_off_slug}/responses/{token}"),
            None,
        ),
        (
            Method::POST,
            format!(
                "/public/surveys/{slug}/responses/{}/complete",
                "y".repeat(43)
            ),
            None,
        ),
        (
            Method::PUT,
            format!("/public/surveys/{slug}/responses/{}", "y".repeat(43)),
            answers(QUESTION, json!({"Choice": "Oui"})),
        ),
        (
            Method::POST,
            format!("/public/surveys/{unknown}/responses/{token}/photos"),
            photo.clone(),
        ),
        (
            Method::POST,
            format!("/public/surveys/{slug}/responses/{}/photos", "y".repeat(43)),
            photo,
        ),
        (Method::GET, "/public/surveys".to_owned(), None),
        (Method::GET, "/public/surveys/".to_owned(), None),
    ];

    assert_eq!(reference_status, StatusCode::NOT_FOUND);
    assert!(!reference.is_empty());
    for (method, path, body) in attempts {
        let (status, text) = app.send(method.clone(), &path, &ip, body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path}: {text}");
        assert_eq!(text.as_bytes(), reference.as_bytes(), "{method} {path}");
    }
    for (method, path) in [
        (Method::GET, format!("/public/surveys/{slug}/responses/")),
        (
            Method::POST,
            format!("/public/surveys/{slug}/responses//complete"),
        ),
    ] {
        let (status, text) = app.send(method.clone(), &path, &ip, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {path}: {text}");
        assert!(!text.contains(&slug), "{method} {path}: {text}");
    }
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_closed_public_survey_says_it_is_closed() {
    let app = harness::start().await;
    let ip = client_ip();
    let (id, slug) = public_survey(&app).await;
    let (status, body) = app
        .call(
            Method::POST,
            &app.org(&format!("/surveys/{id}/close")),
            Some(&app.manager),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (read_status, read) = app
        .send(Method::GET, &format!("/public/surveys/{slug}"), &ip, None)
        .await;
    let (start_status, started) = app
        .send(
            Method::POST,
            &format!("/public/surveys/{slug}/responses"),
            &ip,
            None,
        )
        .await;

    assert_eq!(read_status, StatusCode::CONFLICT, "{read}");
    assert_eq!(json_of(&read)["details"]["reason"], "survey_closed");
    assert_eq!(start_status, StatusCode::CONFLICT, "{started}");
    assert_eq!(json_of(&started)["details"]["reason"], "survey_closed");
    app.cleanup().await;
}

async fn photo_survey(app: &App) -> (String, String) {
    let question = "0195f0a0-0000-7000-8000-000000000011";
    let end = "0195f0a0-0000-7000-8000-000000000012";
    let id = app
        .draft_survey(|s| app.org(s), &app.manager, "PUBLIC")
        .await;
    let (status, body) = app
        .call(
            Method::PATCH,
            &app.org(&format!("/surveys/{id}")),
            Some(&app.manager),
            Some(json!({"graph": {
                "nodes": [
                    {"type": "QUESTION", "id": question, "kind": "PHOTO", "label": "Photo du chantier", "required": true},
                    {"type": "END", "id": end}
                ],
                "edges": [{"from": question, "to": end, "branch": "NEXT"}],
                "start": question
            }})),
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
    (
        body["data"]["public_slug"].as_str().unwrap().to_owned(),
        question.to_owned(),
    )
}

#[tokio::test]
#[ignore = "requires live postgres, redis and the object storage"]
async fn a_public_photo_is_normalized_stored_and_answered_by_its_key() {
    let app = harness::start().await;
    let ip = client_ip();
    let (slug, question) = photo_survey(&app).await;
    let token = start(&app, &slug, &ip).await;
    let other = start(&app, &slug, &ip).await;
    let photos = format!("/public/surveys/{slug}/responses/{token}/photos");

    let (status, uploaded) = app
        .send(
            Method::POST,
            &photos,
            &ip,
            Some(("image/jpeg", PHOTO.to_vec())),
        )
        .await;
    let key = json_of(&uploaded)["data"]["key"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let (other_status, other_uploaded) = app
        .send(
            Method::POST,
            &format!("/public/surveys/{slug}/responses/{other}/photos"),
            &ip,
            Some(("image/jpeg", PHOTO.to_vec())),
        )
        .await;
    let foreign = json_of(&other_uploaded)["data"]["key"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let (pdf_status, pdf) = app
        .send(
            Method::POST,
            &photos,
            &ip,
            Some((
                "application/pdf",
                b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n".to_vec(),
            )),
        )
        .await;
    let response = format!("/public/surveys/{slug}/responses/{token}");
    let (foreign_status, refused) = app
        .send(
            Method::PUT,
            &response,
            &ip,
            answers(&question, json!({"Photos": [foreign]})),
        )
        .await;
    let (save_status, saved) = app
        .send(
            Method::PUT,
            &response,
            &ip,
            answers(&question, json!({"Photos": [key]})),
        )
        .await;
    let (complete_status, completed) = app
        .send(Method::POST, &format!("{response}/complete"), &ip, None)
        .await;

    assert_eq!(status, StatusCode::CREATED, "{uploaded}");
    assert!(
        key.contains(&format!("survey-{}/", app.organization_id)),
        "{key}"
    );
    assert!(key.ends_with(".jpg"), "{key}");
    assert_eq!(other_status, StatusCode::CREATED, "{other_uploaded}");
    assert_eq!(pdf_status, StatusCode::UNPROCESSABLE_ENTITY, "{pdf}");
    assert_eq!(
        foreign_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{refused}"
    );
    assert_eq!(
        json_of(&refused)["details"]["reason"],
        "answer_foreign_photo"
    );
    assert_eq!(save_status, StatusCode::OK, "{saved}");
    assert_eq!(complete_status, StatusCode::OK, "{completed}");
    assert_eq!(
        json_of(&completed)["data"]["answers"][&question]["Photos"][0],
        key
    );
    let (_, resumed_other) = app
        .send(
            Method::GET,
            &format!("/public/surveys/{slug}/responses/{other}"),
            &ip,
            None,
        )
        .await;
    for id in [
        json_of(&completed)["data"]["id"].clone(),
        json_of(&resumed_other)["data"]["id"].clone(),
    ] {
        let id = id.as_str().expect("a response has an id").to_owned();
        let (status, body) = app
            .call(
                Method::DELETE,
                &app.org(&format!("/survey-responses/{id}")),
                Some(&app.manager),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
        assert_eq!(app.recorded_photos(&id).await, 0);
    }
    app.cleanup().await;
}

async fn limited_after(
    app: &App,
    method: Method,
    path: &str,
    body: Option<(&'static str, Vec<u8>)>,
    limit: u32,
) {
    let ip = client_ip();
    for call in 1..=limit {
        let (status, text) = app.send(method.clone(), path, &ip, body.clone()).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "call {call} of {limit}: {text}"
        );
    }
    let (status, text) = app.send(method.clone(), path, &ip, body.clone()).await;
    let (other_status, _) = app.send(method, path, &client_ip(), body).await;

    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "call {}: {text}",
        limit + 1
    );
    assert_eq!(json_of(&text)["code"], "E_TOO_MANY_REQUESTS");
    assert_eq!(other_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_sixty_first_start_of_a_minute_from_one_address_is_refused() {
    let app = harness::start().await;
    limited_after(
        &app,
        Method::POST,
        "/public/surveys/inconnu-00000000000000/responses",
        None,
        60,
    )
    .await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_thirty_first_photo_of_a_minute_from_one_address_is_refused() {
    let app = harness::start().await;
    limited_after(
        &app,
        Method::POST,
        &format!(
            "/public/surveys/inconnu-00000000000000/responses/{}/photos",
            "y".repeat(43)
        ),
        Some(("image/jpeg", PHOTO.to_vec())),
        30,
    )
    .await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_six_hundred_and_first_read_of_a_minute_from_one_address_is_refused() {
    let app = harness::start().await;
    limited_after(
        &app,
        Method::GET,
        "/public/surveys/inconnu-00000000000000",
        None,
        600,
    )
    .await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_survey_flooded_from_rotating_addresses_is_refused_after_six_hundred_starts() {
    let app = harness::start().await;
    let slug = format!("inconnu-{}", Uuid::now_v7().simple());
    let path = format!("/public/surveys/{slug}/responses");
    for call in 1..=600 {
        let (status, text) = app.send(Method::POST, &path, &client_ip(), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "call {call}: {text}");
    }

    let (status, text) = app.send(Method::POST, &path, &client_ip(), None).await;
    let (other_status, _) = app
        .send(
            Method::POST,
            &format!(
                "/public/surveys/inconnu-{}/responses",
                Uuid::now_v7().simple()
            ),
            &client_ip(),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{text}");
    assert_eq!(json_of(&text)["code"], "E_TOO_MANY_REQUESTS");
    assert_eq!(other_status, StatusCode::NOT_FOUND);
    app.cleanup().await;
}
