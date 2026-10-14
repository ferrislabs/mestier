use crate::harness;

fn recap_url(app: &harness::App, query: &str) -> String {
    format!(
        "{}/api/v1/organizations/{}/maintenance/recap.pdf{query}",
        app.base_url, app.organization_id
    )
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_recap_pdf_needs_planning_manage_and_a_valid_week() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let anonymous = client
        .get(recap_url(&app, "?week=2026-W43"))
        .send()
        .await
        .expect("anonymous");
    assert_eq!(anonymous.status(), 401);

    let without_permission = client
        .get(recap_url(&app, "?week=2026-W43"))
        .bearer_auth(&app.no_permission_token)
        .send()
        .await
        .expect("no permission");
    assert_eq!(without_permission.status(), 403);

    let bad_week = client
        .get(recap_url(&app, "?week=2026-43"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("bad week");
    assert_eq!(bad_week.status(), 400);

    let pdf = client
        .get(recap_url(&app, "?week=2026-W43"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("pdf");
    assert_eq!(pdf.status(), 200);
    assert_eq!(pdf.headers()["content-type"], "application/pdf");
    assert!(pdf.bytes().await.expect("pdf body").starts_with(b"%PDF"));

    app.cleanup().await;
}
