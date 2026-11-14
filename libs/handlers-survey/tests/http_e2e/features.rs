use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

use crate::harness::{self, App};

async fn toggle(app: &App, path: &str, token: &str, enabled: bool) -> (StatusCode, Value) {
    app.call(
        Method::PUT,
        path,
        Some(token),
        Some(json!({ "enabled": enabled })),
    )
    .await
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_admin_lists_the_self_service_features_with_their_state() {
    let app = harness::start_with_organizations().await;

    let (status, body) = app
        .call(
            Method::GET,
            &app.flag_off_org("/features"),
            Some(&app.flag_off_admin),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["data"]["self_service"],
        json!([{
            "key": "surveys",
            "label": "Questionnaires",
            "description": "Questionnaires et campagnes de réponses, ouverts au public ou aux membres.",
            "enabled": false,
        }])
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn enabling_surveys_opens_the_api_and_disabling_hides_it_without_losing_data() {
    let app = harness::start_with_organizations().await;
    let manager = Some(app.flag_off_manager.as_str());

    let (status, body) = toggle(
        &app,
        &app.flag_off_org("/features/surveys"),
        &app.flag_off_admin,
        true,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["data"],
        json!({"key": "surveys", "enabled": true, "changed": true})
    );
    let (status, listed) = app
        .call(Method::GET, &app.flag_off_org("/features"), manager, None)
        .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["data"]["features"], json!(["surveys"]));
    assert_eq!(listed["data"]["self_service"][0]["enabled"], json!(true));
    let survey = app
        .draft_survey(
            |suffix| app.flag_off_org(suffix),
            &app.flag_off_manager,
            "MEMBERS",
        )
        .await;

    let (status, body) = toggle(
        &app,
        &app.flag_off_org("/features/surveys"),
        &app.flag_off_admin,
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, hidden) = app
        .call(Method::GET, &app.flag_off_org("/surveys"), manager, None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{hidden}");

    toggle(
        &app,
        &app.flag_off_org("/features/surveys"),
        &app.flag_off_admin,
        true,
    )
    .await;
    let (status, kept) = app
        .call(
            Method::GET,
            &app.flag_off_org(&format!("/surveys/{survey}")),
            manager,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{kept}");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_manage_org_is_forbidden_to_toggle() {
    let app = harness::start_with_organizations().await;

    let (status, body) = toggle(
        &app,
        &app.flag_off_org("/features/surveys"),
        &app.flag_off_manager,
        true,
    )
    .await;
    let (_, survey_list) = app
        .call(
            Method::GET,
            &app.flag_off_org("/surveys"),
            Some(&app.flag_off_manager),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(survey_list["status"], json!(404), "{survey_list}");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_staff_only_or_unknown_flag_cannot_be_toggled_by_an_organization() {
    let app = harness::start_with_organizations().await;

    let (drive_ai, drive_body) = toggle(
        &app,
        &app.flag_off_org("/features/drive_ai"),
        &app.flag_off_admin,
        true,
    )
    .await;
    let (unknown, unknown_body) = toggle(
        &app,
        &app.flag_off_org("/features/inconnu"),
        &app.flag_off_admin,
        true,
    )
    .await;
    let (_, listed) = app
        .call(
            Method::GET,
            &app.flag_off_org("/features"),
            Some(&app.flag_off_admin),
            None,
        )
        .await;

    assert_eq!(drive_ai, StatusCode::NOT_FOUND, "{drive_body}");
    assert_eq!(unknown, StatusCode::NOT_FOUND, "{unknown_body}");
    assert_eq!(listed["data"]["features"], json!([]));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn one_organization_cannot_toggle_the_features_of_another() {
    let app = harness::start_with_organizations().await;

    let (status, body) = toggle(
        &app,
        &app.other_org("/features/surveys"),
        &app.flag_off_admin,
        false,
    )
    .await;
    let (_, still_open) = app
        .call(
            Method::GET,
            &app.other_org("/surveys"),
            Some(&app.other_manager),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert!(still_open["data"].is_array(), "{still_open}");
    app.cleanup().await;
}
