use serde_json::Value;

use crate::{harness, issuer};

const MESSAGE: &str = "L'accès par IA (MCP) est désactivé pour cette organisation.";

fn routes(app: &harness::App) -> [String; 2] {
    [
        format!(
            "{}/api/v1/organizations/{}/does",
            app.base_url, app.organization_id
        ),
        format!(
            "{}/api/v1/organizations/{}/projects",
            app.base_url, app.organization_id
        ),
    ]
}

async fn set_mcp_enabled(app: &harness::App, enabled: bool) {
    sqlx::query("UPDATE organizations SET mcp_enabled = $2 WHERE id = $1")
        .bind(app.organization_id)
        .bind(enabled)
        .execute(&app.pool)
        .await
        .expect("flip the organization's mcp switch");
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_mcp_12_a_delegated_token_is_refused_on_an_organization_without_mcp() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let delegated = issuer::mint_delegated(&app.manager_sub);

    for url in routes(&app) {
        let refused = client
            .get(&url)
            .bearer_auth(&delegated)
            .send()
            .await
            .expect("the api answers the delegated call");
        assert_eq!(refused.status(), 403, "{url}");
        let body: Value = refused.json().await.expect("the refusal is json");
        assert_eq!(body["message"], MESSAGE, "{body}");

        let normal = client
            .get(&url)
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the normal call");
        assert_eq!(
            normal.status(),
            200,
            "a non-delegated token must keep working: {url}"
        );
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_mcp_12_enabling_mcp_lets_a_delegated_token_through() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let delegated = issuer::mint_delegated(&app.manager_sub);

    set_mcp_enabled(&app, true).await;
    for url in routes(&app) {
        let allowed = client
            .get(&url)
            .bearer_auth(&delegated)
            .send()
            .await
            .expect("the api answers the delegated call");
        assert_eq!(allowed.status(), 200, "{url}");
    }

    set_mcp_enabled(&app, false).await;
    for url in routes(&app) {
        let refused = client
            .get(&url)
            .bearer_auth(&delegated)
            .send()
            .await
            .expect("the api answers the delegated call");
        assert_eq!(refused.status(), 403, "{url}");
    }

    app.cleanup().await;
}
