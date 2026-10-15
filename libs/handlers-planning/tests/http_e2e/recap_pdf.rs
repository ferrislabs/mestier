use lopdf::Document;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    harness,
    maintenance_contract::{cleanup, create_contract, seed_site_in},
};

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

fn windows(label: &str, from_week: u8, to_week: u8) -> Value {
    json!({
        "label": label,
        "cadence": { "kind": "WINDOWS", "windows": [{ "from_week": from_week, "to_week": to_week }] }
    })
}

fn text_of(bytes: &[u8]) -> String {
    let doc = Document::load_mem(bytes).expect("the body is a readable PDF");
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    doc.extract_text(&pages).expect("the PDF has text")
}

/// `@spec-ent-12`, against the real recap rather than the sample data: the
/// PDF lists the sites due in the week and none of the others.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_recap_pdf_lists_the_contracts_due_that_week_and_no_other() {
    let app = harness::start().await;
    let due_site = seed_site_in(&app, "GAUME", "Domaine Gaume", "Saint-Pourçain-sur-Sioule").await;
    let later_site = seed_site_in(&app, "ROUSSEAU", "Clos Rousseau", "Cusset").await;
    sqlx::query(
        "INSERT INTO customer_contacts (id, customer_id, first_name, last_name, phone, is_primary) \
         VALUES ($1, $2, 'Odette', 'Gaume', '04 70 00 00 00', true)",
    )
    .bind(Uuid::now_v7())
    .bind(due_site.customer_id)
    .execute(&app.pool)
    .await
    .expect("seed the primary contact");

    create_contract(&app, &due_site, vec![windows("Taille des haies", 43, 45)]).await;
    create_contract(&app, &later_site, vec![windows("Tonte du verger", 46, 48)]).await;

    let pdf = reqwest::Client::new()
        .get(recap_url(&app, "?week=2026-W43"))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("pdf");
    assert_eq!(pdf.status(), 200);
    let text = text_of(&pdf.bytes().await.expect("pdf body"));

    for expected in [
        "Domaine Gaume",
        "Saint-Pourçain-sur-Sioule",
        "Taille des haies",
        "Odette Gaume",
    ] {
        assert!(text.contains(expected), "{expected:?} missing from {text}");
    }
    for unexpected in ["Rousseau", "Cusset", "Tonte du verger"] {
        assert!(
            !text.contains(unexpected),
            "{unexpected:?} should not be in {text}"
        );
    }

    sqlx::query(
        "DELETE FROM customer_contacts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
    )
    .bind(app.organization_id)
    .execute(&app.pool)
    .await
    .expect("clear the contacts");
    cleanup(&app).await;
}
